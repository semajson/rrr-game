var login_form = document.getElementById("joinForm");

function joinGame(event) {
  event.preventDefault();
  const game_id = document.getElementById("joinGameId").value;

  fetch("http://localhost:7878/rrr-game", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: "",
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
      window.location.replace("/menu.html");
    })
    .catch((error) => {
      console.error("Error is:", error);
    });
}

login_form.addEventListener("submit", doLogin);
