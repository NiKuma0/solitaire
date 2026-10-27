"use strict";
// Kept out of index.html so the CSP needs no 'unsafe-inline' for scripts.

// An opaque canvas: miniquad writes colours that are not premultiplied, so a
// transparent one would show anything translucent too bright.
const getContext = HTMLCanvasElement.prototype.getContext;
HTMLCanvasElement.prototype.getContext = function (type, attributes) {
  return getContext.call(this, type, { ...attributes, alpha: false });
};

document.getElementById("glcanvas").focus();
load("solitaire.wasm");
