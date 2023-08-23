/* global isX */

function joinConn() {
  const tempId = document.getElementById("connInput").value;
  if (tempId == "") {
    return;
  }
  id = tempId;

  document.getElementById("connInput").disabled = true;
  document.getElementById("connBtn").disabled = true;

  socket.emit("room", tempId);
}

socket.on("room", function (msg) {
  console.log("Join Room Message: " + msg);
  if (msg == "win") {
    socket.emit("msg", "conn");
    socket.on("msg", handleWebInput);
  } else {
    if (msg == "full") {
      document.getElementById("showConnErr").textContent =
        "That game appears to already be full. Are you sure you typed it correctly?";
    } else {
      document.getElementById("showConnErr").textContent =
        "Hm... we can't seem to connect to that game code. Are you sure you typed it correctly?";
    }
    document.getElementById("showConnErr").style.display = "block";
    document.getElementById("connBtn").disabled = false;
    document.getElementById("connInput").disabled = false;
  }
});

socket.on("end", function (msg) {
  console.log("Someone left: " + msg);
  disconnect();
});

function sendWebInput(data) {
  socket.emit("msg", data);
}

//Not Host
function handleWebInput(data) {
  console.log("Data received: ");
  console.log(data);

  const e = data.split(":");

  switch (e[0]) {
    case "connGo":
      //Game starting
      isX = e[1] !== "true";
      showSpin(isX);
      return;
    case "getMove":
      const a = parseInt(e[1]);
      const b = parseInt(e[2]);
      const i = parseInt(e[3]);
      const j = parseInt(e[4]);
      const lastTurn = e[5] === "1";
      const state = JSON.parse(data.substring(18));
      board.move(lastTurn, state, a, b, i, j);
      updateInfoBox(state);
      return;
    case "wantRestart":
      oneRematch(false);
      return;
    case "restarting":
      newMatch();
      return;
  }

  console.log("Data could not be recognized");
}

function startGame() {
  setupGameDisplay();
}

function clickedCell(a, b, i, j) {
  sendWebInput(`click:${a}:${b}:${i}:${j}`);
}

function clickRematch() {
  oneRematch(true);
  sendWebInput("wantRematch");
}

///////////////////////////////////////////////////////////////////////////////

/**
 * Hide the connection information and display the turn spinner. Calls function
 * {@link startGame} after spinner finishes.
 *
 * @param {boolean} isX Whether the player is player X
 */
function showSpin(isX) {
  document.getElementById("connSetup").style.display = "none";
  document.getElementById("connSpin").style.display = "block";

  const _a = window
    .getComputedStyle(document.getElementById("connSpin1"))
    .getPropertyValue("opacity");
  const _b = window
    .getComputedStyle(document.getElementById("connSpin2"))
    .getPropertyValue("opacity");
  if (_a != 1 || _b != 1) {
    console.log("Weird");
  }

  if (isX) {
    document.getElementById("connSpin1").style.transform = "rotateY(0deg)";
    document.getElementById("connSpin2").style.transform = "rotateY(-180deg)";
  } else {
    document.getElementById("connSpin1").style.transform = "rotateY(180deg)";
    document.getElementById("connSpin2").style.transform = "rotateY(0deg)";
  }

  window.setTimeout(startGame, 8000);
}

const messages = {
  youX: "You are $X$ - ",
  youO: "You are $O$ - ",
  youNext: "Your Move",
  notNext: "Please Wait",
  anyMove: "Please make a move in any area",
  restrictMove: "Please make a move in the indicated area",
  waitMove: "Waiting on your opponent to make their move",
  youWin: "You Win",
  youLose: "You Lose",
  tie: "Tie",
};
// noinspection JSUnresolvedFunction
/**
 * The socket.io connection to the server
 */
const socket = io("https://xingnode.azurewebsites.net/uttt");

/**
 * The ID of the game
 * @type{string}
 */
let id;

/**
 * The game board
 * @type {GameBoard}
 */
let board;

/**
 * The info box
 * @type {InfoBox}
 */
let info;

/**
 * Whether the host is player X
 * @type {boolean}
 */
let isX;

/**
 * Initialize the game board and info box, as well as hiding everything else.
 */
function setupGameDisplay() {
  board = new GameBoard();
  info = new InfoBox();
  document.getElementById("connSpin").style.display = "none";
  document.getElementById("boardWrap").style.display = "";
  document.getElementById("infoBox").style.display = "block";
  document.getElementById("infoBoxWrap").style.display = "inline-block";

  if (isX) {
    setMsg(messages.youNext, messages.anyMove);
    board.unlockBoard();
  } else {
    setMsg(messages.notNext, messages.waitMove);
    board.lockBoard();
  }
}

/**
 * Sets the info box display message with an indicator for which player is up
 *
 * @param heading The header message to display after the "You are X/O - "
 * @param msg The small text to display
 */
function setMsg(heading, msg) {
  if (isX) {
    info.setMsg(messages.youX + heading, msg);
  } else {
    info.setMsg(messages.youO + heading, msg);
  }
}

/**
 * Updates the info box after a turn was made
 *
 * @param {GameState} state The new, post-turn game state
 */
function updateInfoBox(state) {
  if (!state.status) {
    // noinspection JSIncompatibleTypesComparison
    if (state.status === null) {
      // Tie
      setMsg(messages.tie, null);
    } else {
      // Winner
      setMsg(state.turn === isX ? messages.youWin : messages.youLose, null);
    }
    document.getElementById("rematchBox").style.display = "block";
  } else {
    if (state.turn === isX) {
      board.unlockBoard();
      setMsg(
        messages.youNext,
        state.lastMove ? messages.restrictMove : messages.anyMove
      );
    } else {
      board.lockBoard();
      setMsg(messages.notNext, messages.waitMove);
    }
  }
}

/**
 * End the game after one opponent disconnects
 */
function disconnect() {
  if (board === null) {
    alert("Your opponent disconnected");
    return;
  }
  board.lockBoard();
  info.setMsg("Your Opponent Disconnected", null);
}

/**
 * Update the rematch display to reflect that at least one person would like a rematch
 *
 * @param {boolean} isYou Whether you were the player wanting a rematch
 */
function oneRematch(isYou) {
  if (isYou) {
    document.getElementById("rematchBtn").disabled = true;
  } else {
    document.getElementById("rematchMsg").style.display = "block";
  }
}

/**
 * Reset the board for a new match
 */
function newMatch() {
  document.getElementById("rematchBox").style.display = "none";
  document.getElementById("rematchMsg").style.display = "none";
  document.getElementById("rematchBtn").disabled = false;
  board.reset();
  isX = !isX;
  if (isX) {
    setMsg(messages.youNext, messages.anyMove);
    board.unlockBoard();
  } else {
    setMsg(messages.notNext, messages.waitMove);
    board.lockBoard();
  }
}
