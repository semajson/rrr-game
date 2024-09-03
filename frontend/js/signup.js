// var login_form = document.getElementById("loginForm");

// console.log("loaded");
// // alert("trying login");

// function doLogin(event) {
//   event.preventDefault();
//   const username = document.getElementById("loginUsername").value;
//   const password = document.getElementById("loginPassword").value;

//   fetch("http://localhost:7878/users", {
//     method: "POST",
//     headers: {
//       "Content-Type": "application/json",
//     },
//     body: JSON.stringify({
//       username: username,
//       password: password,
//     }),
//   })
//     .then((response) => {
//       console.log("Response is" + response);
//     })
//     .catch((error) => {
//       console.error("Error:", error);
//     });

//   alert("trying login");
// }

// login_form.addEventListener("submit", doLogin);
