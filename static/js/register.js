window.onload = function() {
    const { startRegistration } = SimpleWebAuthnBrowser;

    // Color chooser
    const colorButtons = document.getElementsByClassName("color-button");
    const icon = document.getElementById("color-preview");

    for (let i = 0; i < colorButtons.length; i++) {
        colorButtons[i].addEventListener("click", function() {
            const color = this.getAttribute("data-color");
            icon.className = "transition shield mb-1 mt-2 ~" + color;
            icon.dataset.color = color;
        });
    }

    // Input validation
    function validate() {

        let didError = false;
        const name = document.getElementById("name-field");
        const email = document.getElementById("email-field");

        const nameError = document.getElementById("name-error");
        const emailError = document.getElementById("email-error");
        nameError.innerText = "";

        if (email != null) {
            emailError.innerText = "";

            if (email.value.split("@").length !== 2) {
                emailError.innerText = "Error: Invalid email address";
                didError = true;
            }

            if (email.value.length === 0) {
                emailError.innerText = "Error: E-mail cannot be empty"
                didError = true;
            }
        }

        if (name.value.length === 0) {
            nameError.innerText = "Error: Passkey name cannot be empty"
            didError = true;
        }

        return !didError;
    }

    // Webauthn registration
    const button = document.getElementById("register-button");
    const error = document.getElementById("button-error");

    const name = document.getElementById("name-field");

    async function attemptRegistration() {
        button.classList.add("loading");
        error.classList.add("invisible");
        if (validate()) {
            const email = document.getElementById("email-field");
            const link = document.getElementById("link-id");

            let params = "";
            if (email != null && link != null) {
                params = "?email=" + email.value + "&registration_id=" + link.getAttribute("data-link")
            }

            const resp = await fetch("/register_key/start" + params, {method: "POST"});
            let attResp;
            try {
                const jsonResp = await resp.json();
                attResp = await startRegistration(jsonResp["publicKey"]);
            } catch (e) {
                error.classList.remove("invisible");
                if (e.name === "NotAllowedError") {
                    error.innerText = "Error: Key registration was either cancelled or timed out";
                } else {
                    error.innerText = e;
                }
                button.classList.remove("loading");
                return;
            }

            const keyName = encodeURIComponent(name.value);
            const keyColor = encodeURIComponent(icon.dataset.color);
            const url = "/register_key/finish?name=" + keyName + "&color=" + keyColor;

            const verificationResp = await fetch(url , {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(attResp),
            });

            if (verificationResp.status === 200) {
                window.location.href = "/";
            } else {
                error.classList.remove("invisible");
                error.innerText = "Error: Key registration failed";
            }
        }
        button.classList.remove("loading");
    }

    button.addEventListener("click", attemptRegistration);
}