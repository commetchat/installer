import particles from 'particlesjs'


if (!('remove' in Element.prototype)) {
    Element.prototype.remove = function () {
        if (this.parentNode) {
            this.parentNode.removeChild(this);
        }
    };
}

window.onload = function () {
    particles.init({
        selector: '.tsparticles',
        color: "#FFFFFF",
        speed: 0.1,
        sizeVariations: 1,
        minDistance: 20,
    });

};

window.setText = function (text) {
    var textElement = document.getElementById("text");
    textElement.textContent = decodeURIComponent(text);
}

window.start = function () {

    var button = document.getElementById("install_button");
    button.remove()

    external.invoke("start");
}