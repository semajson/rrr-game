const tileSize = 16;

// Setup
const username = sessionStorage.getItem("username");
const create_game_rsp = JSON.parse(localStorage.getItem("initialGamestate"));
// console.log("create_game_rsp is: " + JSON.stringify(create_game_rsp));
const initial_user_coord = create_game_rsp.user_coord;
const game_id = create_game_rsp.game_id;
let gamestate = create_game_rsp.visible_gamestate;

const canvas = document.getElementById("gameCanvas");
const context = canvas.getContext("2d");
context.canvas.width = tileSize * gamestate.terrain[0].length;
context.canvas.height = tileSize * gamestate.terrain.length;
let gameImages = null;

// Entry point
loadGameImages().then((foundGameImages) => {
  gameImages = foundGameImages;
  // Draw initial game (todo might be bad)
  drawTerrain(gamestate.terrain, gameImages);
  drawUser(initial_user_coord, gamestate.top_left_coord, gameImages);

  // Setup regular tick
  setInterval(gameTick, 3000);
});

// Main loop
async function gameTick() {
  console.log("Game tick");
  gamestate = await getGamestate();

  drawTerrain(gamestate.terrain, gameImages);
  drawUser(gamestate.users[username], gamestate.top_left_coord, gameImages);
}

function loadGameImages() {
  const gameImages = {
    // Tiles
    tile_G: new Image(),
    tile_R: new Image(),
    tile_W: new Image(),

    // Sprites
    user: new Image(),
  };

  gameImages["tile_G"].src = "images/grass.png";
  gameImages["tile_R"].src = "images/rock.png";
  gameImages["tile_W"].src = "images/water.png";
  gameImages["user"].src = "images/user.png";

  // Return a promise that resolves when all images are loaded
  return Promise.all([
    new Promise((resolve) => (gameImages["tile_G"].onload = resolve)),
    new Promise((resolve) => (gameImages["tile_R"].onload = resolve)),
    new Promise((resolve) => (gameImages["tile_W"].onload = resolve)),
    new Promise((resolve) => (gameImages["user"].onload = resolve)),
  ]).then(() => {
    return gameImages;
  });
}

function drawTerrain(board, gameImages) {
  for (let row = 0; row < board.length; row++) {
    for (let col = 0; col < board[row].length; col++) {
      const tileType = board[row][col];
      const tileImage = gameImages["tile_" + tileType];
      context.drawImage(
        tileImage,
        col * tileSize,
        row * tileSize,
        tileSize,
        tileSize
      );
    }
  }
}

function drawUser(userCoord, topLeftCoord, gameImages) {
  console.log("User coord is " + JSON.stringify(gamestate));
  const row = userCoord.x - topLeftCoord.x;
  const col = userCoord.y - topLeftCoord.y;
  const userImage = gameImages["user"];

  context.drawImage(
    userImage,
    col * tileSize,
    row * tileSize,
    tileSize,
    tileSize
  );
}

async function getGamestate() {
  const token = sessionStorage.getItem("token");

  // This feels wrong, as getting user coord from the old gamestate!
  // This will break when go off edge of map.
  // Maybe want to store a local usercoord?
  const users = gamestate.users;
  const user_coord = users[username];

  console.log("Found user coord is :" + user_coord.x + ", " + user_coord.y);

  const response = await fetch(
    "http://localhost:7878/rrr-game/" +
      game_id +
      "?x=" +
      user_coord.x +
      "&y=" +
      user_coord.y,
    {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token,
      },
    }
  );
  const data = await response.json();

  return data;
}
