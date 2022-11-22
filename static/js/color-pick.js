window.onload = function() {
    // Color chooser
    const colorButtons = document.getElementsByClassName("color-button");
    const preview = document.getElementById("color-preview");

    for (let i = 0; i < colorButtons.length; i++) {
        colorButtons[i].addEventListener("click", function () {
            const color = this.getAttribute("data-color");
            preview.className = preview.className.replace(/\w*~\w*/g, "~" + color);
            preview.dataset.color = color;
        });
    }
}