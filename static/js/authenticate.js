window.onload = function() {
    const { startAuthentication } = SimpleWebAuthnBrowser;

    // Webauthn authentication
    const button = document.getElementById("auth-button");
    const error = document.getElementById("button-error");

    const redirect = document.getElementById("redirect-id");

    async function attemptAuthentication() {
        button.classList.add("loading");
        const resp = await fetch("/authenticate/start?email=videah@selfish.systems", { method: "POST" });
        let asseResp;
        try {
            const jsonResp = await resp.json();
            asseResp = await startAuthentication(jsonResp["publicKey"])
        } catch (error) {

        }

        const verificationResp = await fetch("/authenticate/finish", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(asseResp),
        });

        if (verificationResp.status === 200) {
            if (redirect) {
                if (redirect.dataset.link) {
                    window.location.href = redirect.dataset.link;
                } else {
                    document.location.href = "/";
                }
            } else {
                document.location.href = "/";
            }
        }
        button.classList.remove("loading");
    }

    button.addEventListener("click", attemptAuthentication);
}