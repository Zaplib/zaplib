# Language idea: Typescript++

_JP Posma, April 2022_

Typescript and Rust feel pretty similar. In some ways, Rust feels like a more restrictive but faster version of Typescript. But Typescript/Javascript can be very fast too; owing to years of hard work by browser vendors. In this article we'll look at the performance characteristics in more detail, and if we can have the best of both worlds.

## JS vs Wasm performance

[Various](https://medium.com/@torch2424/webassembly-is-fast-a-real-world-benchmark-of-webassembly-vs-es6-d85a23f8e193) [articles](https://javascript.plainenglish.io/webassembly-vs-javascript-can-wasm-beat-javascript-in-benchmark-cd7c30faaf7a) [have](https://github.com/marcomontalbano/wasm-vs-js-benchmark) [been](https://betterprogramming.pub/how-fast-is-webassembly-versus-javascript-bc0eca058a54) [written](https://takahirox.github.io/WebAssembly-benchmark/), [comparing](https://engineering.widen.com/blog/A-Tale-of-Performance-Javascript,-Rust,-and-WebAssembly/) [the](https://weihang-wang.github.io/papers/imc21.pdf) [performance](https://news.ycombinator.com/item?id=23776976) [of](https://www.youtube.com/watch?v=uMuYaES4W3o) [Javascript](http://8bitworkshop.com/docs/posts/2021/webassembly-vs-javascript-emulator-performance.html) [versus](https://medium.com/samsung-internet-dev/performance-testing-web-assembly-vs-javascript-e07506fd5875) [WebAssembly](https://pspdfkit.com/blog/2018/a-real-world-webassembly-benchmark/) [("Wasm")](https://www.diva-portal.org/smash/get/diva2:1640220/FULLTEXT01.pdf) [in](https://trepo.tuni.fi/bitstream/handle/10024/120030/YleniusSamuli.pdf?sequence=2) [the](https://daninet.github.io/hash-wasm-benchmark/) [browser](https://github.com/shamadee/web-dsp). Often with disappointing results for Wasm — we'd [expect Wasm to be blazing fast](https://news.ycombinator.com/item?id=13605213), but often it's just a little bit faster, or even slower. It's rare to see a benchmark where Wasm outperforms JS by more than 2x, which is nice, but often not enough to convince people to fully move over to Wasm.

To be fair, WebAssembly is a much younger technology than Javascript. Both the implementation and the APIs of WebAssembly have a lot of room to grow. It simply hasn't had the time and resources that JavaScript & V8 have had to be so damn fast! While we can only look at the benchmarks of today, it's reasonable to hope that there's room for WebAssembly to continue to get faster, even relative to JavaScript. There simply might be more lower hanging fruit left.

Another problem with benchmarks is that they often don't measure realistic scenarios. For example, it's easy to [trick yourself](https://benediktmeurer.de/2016/12/16/the-truth-about-traditional-javascript-benchmarks/#garbage-collection-considered-harmful) into not measuring garbage collection time, by finishing your measurements before the garbage collector can run.

Let's be a bit more precise about comparing JS vs Wasm. First, there are two metrics that are interesting:
1. **Maximum performance.** How fast can a significantly hand-tuned implementation get?
2. **Canonical performance.** How fast can you get using standard language features or libraries?

### Maximum performance

In terms of maximum performance, JS and Wasm are roughly the same (except in a small number of edge cases), because in either it's possible to operate directly on byte arrays (`ArrayBuffer`). In other words, JavaScript is able to simulate a memory managed language if you use `ArrayBuffer` and manually manage your memory:
1. This means that you avoid any garbage collection or Javascript object overhead.
2. You're in full control over allocations / cache locality / etc.
3. Javascript is very fast when just using loops, local variables, arithmethic, function calls, etc.

This is the case that most of the benchmarks mentioned above focus on. Even there, you can find benchmarks which produce spectacular results, like the [hash-wasm benchmark](https://daninet.github.io/hash-wasm-benchmark/), in which the Wasm version is often 10x faster. But in most cases you'll find much more modest results, because the JavaScript benchmarks are manually managing memory (which is the slowest part of JavaScript).

As of current writing, there are a couple of differences in language features and implementations that give either JS or Wasm an edge:
* **JS**: Access to zero-copy native APIs, like `TextEncoder` and `FileReader.readAsArrayBuffer`. In JS you can immediately use the result of these functions, whereas in Wasm you first need to copy the result into the Wasm memory.
* **JS**: Zero-copy multiple memories. You can cheaply create multiple `ArrayBuffer`s, whereas in Wasm there is only a single memory, and growing it is fairly expensive.
* **Wasm**: SIMD instructions. SIMD.js APIs have been [deprecated](https://github.com/tc39/ecmascript_simd) in favour of a Wasm-only implementation.
* **Wasm**: Upfront compiler optimizations. JS can hot-swap in optimized bytecode (even profile-guided optimization based actual program behavior), but this takes a while to kick in. Running such an optimizer also consumes resources. With Wasm you can run an optimizer upfront, and for much longer.

### Canonical performance

Canonical performance might be more interesting to most people. You want the code that is naturally easy to write to be performant. However, even here we have to be careful! For most software, performance follows a power law, where a small fraction of code takes most of the time. In that case it's not a big deal to hand-optimize that part. Canonical performance is more important if such "hot code" is spread out over the entire codebase, which is quite rare.

![](./img/benchmark.gif)


[This 3d character animation benchmark](https://www.lucidchart.com/techblog/2017/05/16/webassembly-overview-so-fast-so-fun-sorta-difficult/) is the best benchmark I've found so far for canonical performance:
1. It's an dual implementation of a relatively complex system in canonical Javascript and canonical JS.
2. It's a system that uses a lot of individual objects in a nester hierarchy; which is pretty representative of many real-world applications.
3. It measures continuously, giving garbage collection no place to hide.
4. It makes you viscerally feel the difference in performance.

5 years ago, when this benchmark was made, Wasm was about 10x faster than JS. On my M1 Mac in 2022 in Chrome, it's only about 5x faster, suggesting that JS has become quite a bit faster since then. But also keep in mind that this was compiled using Emscripten from 5 years ago; that project has also gotten better since then, and can use newer WebAssembly features.

## Simple example

Let's make all these differences a bit more concrete. Let's say that we're writing a function that takes the average length of a bunch of 2d vectors. In Typescript this could look something like this:

```typescript
// Unoptimized Typescript
type Vec2 = { x: number, y: number };

function avgLen(vecs: Vec2[]): number {
    let total = 0;
    for (const vec in vecs) {
        total += Math.sqrt(vec.x*vec.x + vec.y*vec.y);
    }
    return total / vecs.length;
}
```

That isn't too bad by itself, but if these `vecs` get regenerated a lot, then this can cause pretty long garbage collection pauses. Even if these objects are mostly static, they can slow down the garbage collector, since they add to the total number of objects that need to be checked. Finally, if the Javascript compiler can't infer that these objects are always just static objects with `x` and `y` fields, it might fall back to a slower code path where it needs to do an expensive attribute lookup every time. That is to say, the performance of this code might be a bit unpredictable.

It's pretty easy to address these concerns if we're willing to write less canonical code, by storing all `vecs` in a big ArrayBuffer, where every pair of numbers is represented as 16 bytes: first the `x` coordinate as a 64-bit (8-byte) floating point number, followed by the `y` coordinate in the same format:

```typescript
// Optimized Typescript, using ArrayBuffers
function avgLen(vecs: ArrayBuffer): number {
    let total = 0;
    let float64 = new Float64Array(vecs);
    for (let i=0; i<float64.length; i += 2) {
        const x = float64[i];
        const y = float64[i+1];
        total += Math.sqrt(x*x + y*y);
    }
    return total / (float64.length / 2);
}
```

This is less ergonomic, but avoids all of the problems that we had with the canonical implementation. Now, the strength of WebAssembly is that you can write this faster version without having to compromise on ergnomics. For example, the equivalent of this second version in Rust would look like this:

```rust,noplayground
// Unoptimized Rust
struct Vec2 { x: f64, y: f64 }

fn avg_len(vecs: &[Vec2]) -> f64 {
    let mut total = 0.0;
    for vec in vecs {
        total += (vec.x*vec.x + vec.y*vec.y).sqrt();
    }
    return total / vecs.len() as f64;
}
```

## Best of both worlds

With Zaplib we try to make it easier to get the canonical expressiveness and speed of Rust, while embedding it inside an otherwise Javascript-heavy application. However, switching to Rust+Wasm has some significant downsides as well:
1. Rust can be scary! While easier to learn than C++, the learning curve can still be steep, especially when you have to learn to think about ownership and the borrow checker.
2. Toolchain integration can be daunting. You need to set up Rust in development builds, production builds, local testing, continuous integration, and so on.
3. Communicating data between JS and Rust is a bit of work to set up, and can even be expensive, if you need to copy data back and forth a lot.
4. People have often already invested a lot into JS optimizations, and have developed expertise.

With Zaplib we try to make all of this easier; e.g. for (1) we make the Rust APIs in our framework as easy to use as possible; for (2) we've already built Webpack and Node.js integrations and are planning more; and for (3) we're building a JS-Rust bridge that is hopefully easier to use than [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen). But still, in many cases the juice might not be worth the squeeze, if instead you can just do some local optimizations of the Javascript instead. Even if that makes some parts of the codebase less canonical / ergonomic, it might be the right tradeoff.

### Typescript++

I'm wondering if there is a way to bring some of the benefits of Rust to Typescript, by adding the ergonomics from Rust, but still compiling to Javascript — no Wasm needed. There are different ways we could go about this. For example, you could imagine a syntax very similar to the original Typescript, but which would compile to the optimized Typescript with ArrayBuffers:

```typescript
// Typescript++
type Vec2 = { x: number, y: number };

function avgLen(vecs: ArrayBuffer<Vec2>): number {
    let total = 0;
    for (const vec in vecs) {
        total += Math.sqrt(vec.x*vec.x + vec.y*vec.y);
    }
    return total / vecs.length;
}
```

This would be an extension of the language; so we could call it "Typescript++" or so, since every Typescript program would also be a valid Typescript++ program, but not the other way around. Of course, it is possible to something like this purely with libraries, like [BufferBackedObject](https://github.com/GoogleChromeLabs/buffer-backed-object), but doing this at the language level feels a lot more ergonomic.

This might be a bit of a niche analogy, but this quote about using C-style datatypes in LuaJIT captures it quite well:

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">this is cool (from <a href="https://t.co/rPYJ0nXuLt">https://t.co/rPYJ0nXuLt</a>)<br><br>&quot;I often write my LuaJIT programs from the ground up designed around C data types and C-style memory allocation discipline. But I can always ditch that in areas where I know I don&#39;t care…&quot; <a href="https://t.co/898pcPvhaP">pic.twitter.com/898pcPvhaP</a></p>&mdash; Omar Rizwan (@rsnous) <a href="https://twitter.com/rsnous/status/1309673353045704705?ref_src=twsrc%5Etfw">September 26, 2020</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

### Typescript––

The other option would be to not add to the language, but to instead automatically allocate objects in a big linear `ArrayBuffer`, like in Wasm. This would mean that we'd have to add an ownership model, like Rust's borrow checker, which we could add as a linter. We might still need annotations in the code to specify where ownership is transferred vs borrowed, so maybe some light language additions are necessary, but they could be pretty minimal. And they could also be expressed in Typescripts existing type system, e.g.:

```typescript
// Typescript--
function avgLen(vecs: Borrow<Vec2[]>): number {
```

Overall, this would put restrictions on the language, so we could call this Typescript––, since every Typescript–– program is a valid Typescript program, but not the other way around. In practice, you wouldn't want to do this for your entire program, so you should be able to specify at the file, type, or function level where to use TS–– versus regular TS.

Another idea to make Typescript–– a bit less restrictive and more ergonomic, would be to use reference-counting of objects, and then try to optimize most away using [compile-time reference counting as pioneered by Lobster](https://aardappel.github.io/lobster/memory_management.html). This gives most of the advantages of manually managed memory, but makes the ownership model much easier to read about. It is however slightly less performant, and more importantly, makes performance less predictable, since it becomes more reliant on compiler cleverness.

### Other ideas

Another advantage of both of these ideas is that the memory-managed code could also be used to generate code for a faster language, like Rust, or C. This means that — once you've ported enough of your code — it becomes possible to run this code in other contexts. You could run the same code super fast on the backend, in a native app ([Zapium](./zapium.md), where it would likely run about twice as fast), or even in WebAssembly! This seems like a more viable path towards high-performance frontends than getting people to move to Rust right away.

Libraries are another important consideration. JS already has a huge library ecosystem, and a lot of these libraries use Typescript already. If we can make it so you can still use most of your libraries with TS++ or TS––, then that would help enormously in getting this adopted. And some libraries could even add explicit support, where necessary.

## Conclusion

We've only looked at a simple example, and there's a lot more that needs to be figured out in practice. But this seems like an interesting way to incrementally make webapps faster, while keeping the door open to deeper solutions like Rust and WebAssembly. Or maybe it's totally insane, not practically feasible, etc. If you have thoughts, please let us know; I'm [@JanPaul123](https://twitter.com/JanPaul123) on Twitter, and my cofounder is [@stevekrouse](https://twitter.com/stevekrouse).
