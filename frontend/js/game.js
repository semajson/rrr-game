const tileSize = 16;

// Setup
let gameState = localStorage.getItem("initialGamestate");
gameState = JSON.parse(gameState);

const canvas = document.getElementById("gameCanvas");
const context = canvas.getContext("2d");
context.canvas.width = tileSize * gameState.visible_gamestate.terrain[0].length;
context.canvas.height = tileSize * gameState.visible_gamestate.terrain.length;

// Main loop
loadGameImages().then((gameImages) => {
  // Draw initial game (todo might be bad)
  drawTerrain(gameState.visible_gamestate.terrain, gameImages);
  drawUser(gameState.user_coord, gameState.top_left_visible_coord, gameImages);

  setInterval(gameTick, 3000);
});

async function gameTick() {
  console.log("Game tick");

  gameState = await getGameState();
  console.log("gamestate is: " + JSON.stringify(gameState));

  console.log();
  drawTerrain(gameState.visible_gamestate.terrain, gameImages);
  drawUser(gameState.user_coord, gameState.top_left_visible_coord, gameImages);
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
  const row = userCoord.x - topLeftCoord.x;
  const col = userCoord.y - topLeftCoord.y;

  console.log("col (x) is " + col);
  console.log("row (y) is " + row);

  const userImage = gameImages["user"];

  context.drawImage(
    userImage,
    col * tileSize,
    row * tileSize,
    tileSize,
    tileSize
  );
}

async function getGameState() {
  const token = sessionStorage.getItem("token");

  const response = await fetch(
    "http://localhost:7878/rrr-game/" +
      gameState.game_id +
      "?x=" +
      gameState.user_coord.x +
      "&y=" +
      gameState.user_coord.y,
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
