import * as THREE from 'https://cdn.skypack.dev/three@v0.135.0';
import { OrbitControls } from 'https://cdn.skypack.dev/three@v0.135.0/examples/jsm/controls/OrbitControls'

const loadSTLIntoGeometry = async (assetUrl) => {
    const buffer = await fetch(assetUrl).then(r => r.arrayBuffer());
    const data = new DataView(buffer);

    const HEADER_LENGTH = 80;
    const numTriangles = data.getUint32(HEADER_LENGTH, true);
    const vertices = new Float32Array(numTriangles * 9);
    const normals = new Float32Array(numTriangles * 9);
    for (let i = 0; i < numTriangles; i++) {
        const offset = HEADER_LENGTH + 4 + i * 50;

        const normalX = data.getFloat32(offset, true);
        const normalY = data.getFloat32(offset + 4, true);
        const normalZ = data.getFloat32(offset + 8, true);

        for (let j = i * 9, k = 0; k < 36; j += 3, k += 12) {
            vertices[j] = data.getFloat32(offset + 12 + k, true);
            vertices[j + 1] = data.getFloat32(offset + 16 + k, true);
            vertices[j + 2] = data.getFloat32(offset + 20 + k, true);

            normals[j] = normalX;
            normals[j + 1] = normalY;
            normals[j + 2] = normalZ;
        }
    }
    const geometry = new THREE.BufferGeometry();
    geometry.attributes.position = new THREE.BufferAttribute(vertices, 3);
    geometry.attributes.normal = new THREE.BufferAttribute(normals, 3);
    geometry.attributes.offset = new THREE.InstancedBufferAttribute(new Float32Array([-10, 0, 10]), 1);
    geometry.attributes.color = new THREE.InstancedBufferAttribute(new Float32Array([1, 1, 0, 0, 1, 1, 1, 0, 1]), 3);

    return geometry;
}

const material = new THREE.ShaderMaterial({
    vertexShader: `
    varying vec3 vPos;
    varying vec3 vNormal;
    varying vec3 vColor;
    attribute float offset;
    attribute vec3 color;
    void main() {
        vPos = position;
        vNormal = normal;
        vColor = color;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(vec3(position.x, position.y + offset, position.z),1.0);
    }
    `,
    fragmentShader: `
    varying vec3 vPos;
    varying vec3 vNormal;
    varying vec3 vColor;
    void main() {
        vec3 lightPosition = vec3(20.,0.,30.);
        vec3 lightDirection = normalize(vPos.xyz - lightPosition);
        gl_FragColor = vec4(clamp(dot(-lightDirection, vNormal), 0.0, 1.0) * vColor,1.0);
    }
    `,
});

const init = async () => {
    const div = document.getElementById("root");
    const scene = new THREE.Scene();

    const camera = new THREE.PerspectiveCamera(40, div.offsetWidth / div.offsetHeight, 0.1, 1000);
    camera.position.set(0, -30, 30);

    const renderer = new THREE.WebGLRenderer();
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.setSize(div.offsetWidth, div.offsetHeight);
    div.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);

    const render = () => {
        renderer.render(scene, camera);
    }

    const geometry = await loadSTLIntoGeometry("/zaplib/examples/tutorial_3d_rendering/teapot.stl");
    const mesh = new THREE.InstancedMesh(geometry, material, 3);
    scene.add(mesh);

    function animate() {
        requestAnimationFrame(animate);
        render();
    }
    animate();
}

init();
