import * as THREE from 'https://cdn.skypack.dev/three@v0.135.0';
import { OrbitControls } from 'https://cdn.skypack.dev/three@v0.135.0/examples/jsm/controls/OrbitControls'

const loadSTLIntoGeometry = async (assetUrl) => {
    await zaplib.initialize({ filename: '/target/wasm32-unknown-unknown/debug/tutorial_3d_rendering_step2.wasm' });

    const [vertices, normals] = await zaplib.callRust("parse_stl");
    const geometry = new THREE.BufferGeometry();
    geometry.attributes.position = new THREE.BufferAttribute(vertices, 3);
    geometry.attributes.normal = new THREE.BufferAttribute(normals, 3);

    return geometry;
}

const material = new THREE.ShaderMaterial({
    vertexShader: `
    varying vec3 vPos;
    varying vec3 vNormal;
    void main() {
        vPos = position;
        vNormal = normal;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position,1.0);
    }
    `,
    fragmentShader: `
    varying vec3 vPos;
    varying vec3 vNormal;
    void main() {
        vec3 lightPosition = vec3(20.,0.,30.);
        vec3 lightDirection = normalize(vPos.xyz - lightPosition);
        gl_FragColor = vec4(vec3(clamp(dot(-lightDirection, vNormal), 0.0, 1.0)),1.0);
    }
    `,
});

const init = async () => {
    const div = document.getElementById("root");
    const scene = new THREE.Scene();

    const camera = new THREE.PerspectiveCamera(40, div.offsetWidth / div.offsetHeight, 0.1, 1000);
    camera.position.set(0, -20, 20);

    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(div.offsetWidth, div.offsetHeight);
    div.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);

    const render = () => {
        renderer.render(scene, camera);
    }

    const geometry = await loadSTLIntoGeometry("/zaplib/examples/tutorial_3d_rendering/teapot.stl");
    const mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);

    function animate() {
        requestAnimationFrame(animate);
        render();
    }
    animate();
}

init();
