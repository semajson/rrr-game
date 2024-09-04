var signup_form = document.getElementById("signUpForm");

function doLogin(event) {
  event.preventDefault();
  const username = document.getElementById("signUpUsername").value;
  const password = document.getElementById("signUpPassword").value;
  const password_repeat = document.getElementById("signUpPasswordRepeat").value;
  const email = document.getElementById("signUpEmail").value;

  if (password != password_repeat) {
    alert("Passwords don't match");
    return;
  }

  fetch("http://localhost:7878/users", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      username: username,
      email: email,
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
      console.log(data.access_token);
      data.access_token;
    })
    .catch((error) => {
      console.error("Error is:", error);
    });
}

signup_form.addEventListener("submit", doLogin);
