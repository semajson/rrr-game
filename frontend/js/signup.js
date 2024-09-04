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
      console.log("Response is" + response);
    })
    .catch((error) => {
      console.error("Error is:", error);
    });

  alert("trying login");
}

signup_form.addEventListener("submit", doLogin);
