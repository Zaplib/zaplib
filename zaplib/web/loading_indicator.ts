import { assertNotNull } from "common";

export function addLoadingIndicator(): void {
  const style = document.createElement("style");
  style.innerHTML = `
    .zaplib_loading_indicator {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        color: #666;
        font-size: 40px;
    }
    .zaplib_loading_indicator > span {
        display: inline-block;
        animation-name: wiggle;
        animation-duration: 1000ms;
        animation-iteration-count: infinite;
        animation-timing-function: ease-in-out;
    }
    @keyframes wiggle {
        0% {transform: rotate(0deg);}
        10% {transform: rotate(10deg);}
        30% {transform: rotate(-10deg);}
        50% {transform: rotate(20deg);}
        70% {transform: rotate(-5deg);}
        90% {transform: rotate(2deg);}
        95% {transform: rotate(0deg);}
    }

    .zaplib_loading_indicator > div {
        position: absolute;
        width: max-content;
        left: 50%;
        top: 50%;
        transform: translate(-50%, 40px);
        font-family: Verdana, Arial Black;
        font-weight: bold;
        font-size: 28px;

        background: #222 -webkit-gradient(linear, left top, right top, from(#222), to(#222), color-stop(0.5, #fff)) 0 0 no-repeat;
        background-image: -webkit-linear-gradient(-40deg, transparent 0%, transparent 40%, #fff 50%, transparent 60%, transparent 100%);
        background-size: 200px;
        -webkit-background-clip: text;
        background-clip: text;
        animation-name: shine;
        animation-duration: 1s;
        animation-iteration-count: infinite;
        text-shadow: 0 0px 0px rgba(255, 255, 255, 0.5);
    }
    @keyframes shine {
        0% {
            background-position: -200px 0;
        }
        100% {
            background-position: 250px 0;
        }
    }`;
  document.body.appendChild(style);

  const loadingIndicator = document.createElement("div");
  loadingIndicator.className = "zaplib_loading_indicator";
  loadingIndicator.innerHTML =
    '<span>⚡</span><div style="color: rgba(255, 202, 0, 0.5);">Loading…</div>';
  document.body.appendChild(loadingIndicator);
}
export function removeLoadingIndicator(): void {
  const loaders = document.getElementsByClassName("zaplib_loading_indicator");
  for (let i = 0; i < loaders.length; i++) {
    assertNotNull(loaders[i].parentNode).removeChild(loaders[i]);
  }
}
