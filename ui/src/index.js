import particles from 'particlesjs'

window.onload = function () {
    particles.init({
        selector: '.tsparticles',
        color: "#FFFFFF",
        speed: 0.1,
        sizeVariations: 1,
        minDistance: 20,
    });

    external.invoke("start");
};

window.setText = function (text) {
    var textElement = document.getElementById("text");
    textElement.textContent = decodeURIComponent(text);
}
