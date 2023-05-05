import * as Comlink from 'comlink'

const boardDisplay = document.getElementById('boardDisplay')
const boardStatus = document.getElementById('boardStatus')

const manualMoveInput = document.getElementById('manualMoveInput')
const manualMoveButton = document.getElementById('manualMoveButton')
const manualMoveError = document.getElementById('manualMoveError')

const engineMoveInput = document.getElementById('engineMoveInput')
const engineMoveButton = document.getElementById('engineMoveButton')
const engineMoveError = document.getElementById('engineMoveError')
const engineDepth = document.getElementById('engineDepth')

const fenInput = document.getElementById('fenInput')
const fenButton = document.getElementById('fenButton')
const fenError = document.getElementById('fenError')
const currentFen = document.getElementById('currentFen')

// Protochess object imported from rust
let protochess

// Run init() when the page loads
init()


// INIT FUNCTIONS

async function init() {
  // Create a separate thread from wasm-worker.js and get a proxy to its handler
  const wasm = await Comlink.wrap(
    new Worker(new URL('./wasm-worker.js', import.meta.url), { type: 'module' })
  ).wasm
  
  if (wasm.supportsThreads) {
    console.info('WebAssembly supports threads')
  } else {
    console.warn('WebAssembly does not support threads')
  }
  
  protochess = wasm.wasmObject
  protochess.setNumThreads(1)
  initUI()
}

async function initUI() {
  updateBoard('Ok')
  manualMoveButton.onclick = manualMoveButtonClick
  engineMoveButton.onclick = engineMoveButtonClick
  fenButton.onclick = fenButtonClick
}

async function updateBoard(resultFlag, winner, exploded) {
  boardDisplay.innerHTML = await protochess.toString()
  const stateDiff = await protochess.getStateDiff()
  currentFen.innerHTML = stateDiff.fen
  if (resultFlag !== 'Ok') {
    let winnerString
    if (winner === 'White') {
      winnerString = 'White wins'
    } else if (winner === 'Black') {
      winnerString = 'Black wins'
    } else {
      winnerString = 'Draw'
    }
    boardStatus.innerHTML = resultFlag + ', ' + winnerString
  } else if (stateDiff.inCheck) {
    boardStatus.innerHTML = 'Check!'
  } else {
    boardStatus.innerHTML = ' '
  }
  if (exploded && exploded.length > 0) {
    boardStatus.innerHTML += ' (exploded: ['
    for (let i = 0; i < exploded.length; i++) {
      boardStatus.innerHTML += `(${exploded[i]}), `
    }
    boardStatus.innerHTML += '])'
  }
}
function clearErrors() {
  manualMoveError.innerHTML = ''
  engineMoveError.innerHTML = ''
  fenError.innerHTML = ''
}


// BUTTON CLICK HANDLERS

async function manualMoveButtonClick() {
  clearErrors()
  try {
    // Attempt to make the move
    const {flag, winner, exploded} = await protochess.makeMoveStr(manualMoveInput.value)
    if (flag === 'IllegalMove') {
      throw 'This move is illegal'
    }
    updateBoard(flag, winner, exploded)
  } catch (e) {
    manualMoveError.innerHTML = e
  }
}

async function engineMoveButtonClick() {
  clearErrors()
  engineDepth.innerHTML = 'Searching...'
  try {
    // Attempt to convert the input to a number
    const timeout = Number(engineMoveInput.value)
    if (isNaN(timeout)) {
      throw engineMoveInput.value + ' is not a valid number'
    }
    if (timeout < 0) {
      throw 'Timeout must be >= 0'
    }
    // Get the engine to make a move
    const {moveInfo, evaluation, depth} = await protochess.getBestMoveTimeout(timeout)
    const makeMoveResult = await protochess.makeMove(moveInfo)
    const {flag, winner, exploded} = makeMoveResult
    await updateBoard(flag, winner, exploded)
    
    const toMove = await protochess.playerToMove()
    const absoluteEval = toMove === 0 ? -evaluation : evaluation
    engineDepth.innerHTML = `Done! Search depth: ${depth}, board evaluation: ${absoluteEval}cp`
  } catch (e) {
    engineMoveError.innerHTML = e
  }
}

async function fenButtonClick() {
  clearErrors()
  try {
    // Attempt to set the FEN
    await protochess.loadFen(fenInput.value)
    updateBoard('Ok')
  } catch (e) {
    fenError.innerHTML = e
  }
}
