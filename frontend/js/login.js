var login_form = document.getElementById("loginForm");

function doLogin(event) {
  event.preventDefault();
  const username = document.getElementById("loginUsername").value;
  const password = document.getElementById("loginPassword").value;

  fetch("http://localhost:7878/sessions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      username: username,
      password: password,
    }),
  })
    .then((response) => {
      console.log(response);
      if (!response.ok) {
        response.json().then((data) => {
          console.error("Error body: " + JSON.stringify(data));
          alert("Error: " + data.error_message);
        });
        throw new Error(
          "status: " + response.status + ", errorcode: " + response.statusText
        );
      }

      return response.json();
    })
    .then((data) => {
      console.log(data);
      sessionStorage.setItem("token", data.access_token);
      window.location.replace("/game.html");
    })
    .catch((error) => {
      console.error("Error is:", error);
    });
}

login_form.addEventListener("submit", doLogin);
