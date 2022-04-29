# 🥲 Zaplib post-mortem

Welp, this is a weird blog post! Very unexpectedly, we quickly invalidated some of the core hypothesis that make Zaplib work as a startup.

## Original Pitch

The pitch went like:

1. JS & the browser are slow
2. Incrementally porting JS to Rust/Wasm will speed up your app
3. We’ll land-and-expand from small ports to take over your whole app
4. In the long-run this evolves to a next-gen stack (”Unity for apps”)

We uncovered holes in this story once we started working with our pilot users. We always knew that it would always be *possible* to speed up slow JS within JS. No Rust/Wasm required. Our bet was that it would be 10x more *ergonomic* to speed up your app, incrementally, in Rust. This did not hold up in real-world implementations.

## Initial Users

User 1 - Not only did they get the “whole vision” of eventually porting their whole app to Rust, but they seemed to have incrementally portable speedup opportunities. We took a week to port their simulator to Rust, and had high hopes it would be significantly faster out of the gate. It was 5% faster. When thinking about how to speed it up, the main way is by using faster linear algebra libraries, but those also exist in JS. Rust didn’t help in any meaningful way here.

User 2 - We ported their renderer to our GPU-accelerated 2d renderer. It was excellent! However the win here was due to our renderer being GPU-accelerated, which is due to WebGL, not Rust/Wasm. They were rightfully hesitant to include a whole new Rust toolchain in their codebase, when it wasn’t actually necessary.

User 3 - They could be an excellent user of Zaplib, but not in an incremental way. They might have been a great user if we were targeting greenfield apps, but that is not a good startup strategy, because 1) you need an enormous API surface area right out of the gate, and 2) you can’t work with existing businesses.

User 4 - We saw 10x improvements when we benchmarked our prototypes. However the fact that we were building those prototypes from scratch allowed us to architect them in fundamentally faster way – making them not truly fair apples-to-apples comparisons. In other words, we maybe could’ve gotten similar speedups in a JS rewrite. Another big source of performance gains was from our GPU-accelerated renderer, which doesn’t require Rust/Wasm (just like with User 2). We did see better ergonomics (threading, zero-cost abstractions) and 2x speedups for native builds, but these are good-to-have features that aren’t enough for people to switch to a new stack for.

## JS vs Rust

Rust is faster than JS in some cases, but those cases are rarer than we expected, and the performance gain is on the order of 2x some of the time, not 10x most of the time. The big 10x gains do appear when you really lean on Rust’s zero-cost abstractions — processing a million tiny Rust structs is faster than a million JS objects for reasons of memory layout and avoiding the GC — but this is a rare case, particularly for our incremental story. Without a 10x improvement we can't imagine engineers making the investment to add a whole new experimental tool chain to their stack that their team has to learn and maintain. We wouldn't do it ourselves, and we can't advise anyone else to do it. There are usually simpler ways to find performance improvements than Rust/Wasm.

## But doesn't Figma use Wasm?

Yes, but upon closer inspection it seems that their use of Wasm is more due to historical accidents — wanting to build in C++ to hedge for their native app — than for critical performance needs. Figma files are processed in C++/Wasm, and this is likely a huge speedup, but most of Figma’s performance magic is due to their WebGL renderer.

## Near Pivots

You may be wondering if there’s an opportunity for us to pivot away from Rust/Wasm, and pull out Zaplib’s WebGL renderer into its own framework. That was what was actually useful for some of our users. We are wondering this too! We are doubtful if there is a business opportunity around that, but we are considering it.

As a last-ditch effort, we figured we might as well “launch” on Hacker News and see if any interesting use-cases for Zaplib came out of the woodwork. We were successful getting on HN two days in a row: [Typescript as fast as Rust: Typescript++](https://news.ycombinator.com/item?id=30947680) & [Show HN: Zaplib – Speed up your webapp with Rust+Wasm](https://news.ycombinator.com/item?id=30960509). However all that traffic didn't convert into any real usage at all, even in "toy" settings. We felt this was a fairly damning result.

## What went wrong

JP is embarrassed that it took him a year of working on this to make these discoveries. It goes to show how easy it is to fool yourself with misleading benchmarking and customer interviews. Steve is embarrassed in a similar way. Compose (his last project) had similar enthusiasm from initial users, but ultimately didn’t work as a startup. We both resolve to get better about not fooling ourselves!

## Where to go from here

One amazing outcome of this experience is that we have discovered that we (JP & Steve) love working together, and hope to stick together to work on another project. We’ve brainstormed ~30 ideas, and have taken a handful somewhat seriously.

Thinking that we might as well use our expertise of building high-performance pro tools, JP spent a week looking for a gap in the market for a Figma-like 3d CAD tool, particularly for mechanical engineers, but ultimately decided there wasn’t an obvious opportunity. Solidworks, AutoDesk, etc seem to have it fairly well covered.

Much further afield, Steve spent that week researching [this idea of a consumer lighting company](https://twitter.com/stevekrouse/status/1479911345810251776). Consumer hardware is a difficult business that is way outside our experience and expertise, so we’re not taking it super seriously.

Last week we went out to Miami for Miami Tech Week to generally be inspired, and in particular to see if we can get excited about crypto. It does seem like they need devtools! JP & I haven’t been so excited about web3 so far, but we try to keep an open mind.

Now we’re back in SF and tinkering on a very silly idea to turn Twitter into a CMS. Likely it’s just a ~weeklong hackathon project, but hopefully you’ll see it in our twitter feeds shortly.

If you have any ideas you think would be interesting to us, please let us know! We are in full brainstorming and tinkering mode.

While this isn't what any of us wanted, it is the second best thing to have happened: failing fast is certainly better than failing slowly!

## ❤️🙏

Steve & JP
