const tileSize = 16;

// Setup
let initial_gamestate = localStorage.getItem("initialGamestate");
initial_gamestate = JSON.parse(initial_gamestate);
// console.log(JSON.stringify(initial_gamestate));

const canvas = document.getElementById("gameCanvas");
const context = canvas.getContext("2d");

context.canvas.width =
  tileSize * initial_gamestate.visible_gamestate.terrain[0].length;
context.canvas.height =
  tileSize * initial_gamestate.visible_gamestate.terrain.length;

// Main loop
loadTileImages().then((tileImages) => {
  drawBoard(initial_gamestate.visible_gamestate.terrain, tileImages);
});

function loadTileImages() {
  const tileImages = {
    G: new Image(),
    R: new Image(),
    W: new Image(),
  };

  // Paths to the images
  tileImages["G"].src = "images/grass.png";
  tileImages["R"].src = "images/rock.png";
  tileImages["W"].src = "images/water.png";

  // Return a promise that resolves when all images are loaded
  return Promise.all([
    new Promise((resolve) => (tileImages["G"].onload = resolve)),
    new Promise((resolve) => (tileImages["R"].onload = resolve)),
    new Promise((resolve) => (tileImages["W"].onload = resolve)),
  ]).then(() => {
    return tileImages;
  });
}

function drawBoard(board, tileImages) {
  for (let row = 0; row < board.length; row++) {
    for (let col = 0; col < board[row].length; col++) {
      const tileType = board[row][col];
      const tileImage = tileImages[tileType];
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
