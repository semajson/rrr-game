var join_game_form = document.getElementById("joinForm");

function joinGame(event) {
  event.preventDefault();
  const game_id = document.getElementById("joinGameId").value;

  const token = sessionStorage.getItem("token");

  fetch("http://localhost:7878/rrr-game/" + game_id + "/players", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer " + token,
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
      //   sessionStorage.setItem("token", data.access_token);
      window.location.replace("/menu.html");
    })
    .catch((error) => {
      console.error("Error is:", error);
    });
}

join_game_form.addEventListener("submit", joinGame);
