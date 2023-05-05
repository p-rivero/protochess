import * as Comlink from 'comlink'

const boardDisplay = document.getElementById('boardDisplay')
const boardStatus = document.getElementById('boardStatus')

const manualMoveInput = document.getElementById('manualMoveInput')
const manualMoveButton = document.getElementById('manualMoveButton')
const manualMoveError = document.getElementById('manualMoveError')

const engineMoveInput = document.getElementById('engineMoveInput')
const engineMoveButton = document.getElementById('engineMoveButton')
const engineMoveError = document.getElementById('engineMoveError')
const engineStatus = document.getElementById('engineStatus')
const engineCurrentResult = document.getElementById('engineCurrentResult')

const fenInput = document.getElementById('fenInput')
const fenButton = document.getElementById('fenButton')
const fenError = document.getElementById('fenError')
const currentFen = document.getElementById('currentFen')

// Protochess object imported from rust
let protochess
let protochessMemory
let mvFromPtr
let mvToPtr
let mvPromoPtr
let scorePtr
let depthPtr

// Run init() when the page loads
init()


// INIT FUNCTIONS

async function init() {
  // Create a separate thread from wasm-worker.js and get a proxy to its handler
  const wasm = await Comlink.wrap(
    new Worker(new URL('./wasm-worker.js', import.meta.url), { type: 'module' })
  ).wasm
  
  if (await wasm.supportsThreads) {
    console.info('WebAssembly supports threads')
  } else {
    console.warn('WebAssembly does not support threads')
  }
  
  protochess = wasm.wasmObject
  protochessMemory = new Uint8Array(await wasm.memoryBuffer)
  mvFromPtr = await protochess.getMvFromPtr()
  mvToPtr = await protochess.getMvToPtr()
  mvPromoPtr = await protochess.getMvPromoPtr()
  scorePtr = await protochess.getScorePtr()
  depthPtr = await protochess.getDepthPtr()
  await protochess.setNumThreads(1)
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
    if (winner === 'white') {
      winnerString = 'White wins'
    } else if (winner === 'black') {
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
  engineStatus.innerHTML = 'Searching...'
  try {
    // Attempt to convert the input to a number
    const timeout = Number(engineMoveInput.value)
    if (isNaN(timeout)) {
      throw engineMoveInput.value + ' is not a valid number'
    }
    if (timeout < 0) {
      throw 'Timeout must be >= 0'
    }
    // Get the best move from the engine
    const stopPtr = await protochess.getStopFlagPtr()
    setTimeout(() => protochessMemory[stopPtr] = 1, timeout)
    startUpdatingResultText()
    const {moveInfo, evaluation, depth} = await protochess.getBestMoveTimeout()
    // Play the move
    const makeMoveResult = await protochess.makeMove(moveInfo)
    const {flag, winner, exploded} = makeMoveResult
    await updateBoard(flag, winner, exploded)
    
    const toMove = await protochess.playerToMove()
    const absoluteEval = toMove === 0 ? -evaluation : evaluation
    engineStatus.innerHTML = `Done! Search depth: ${depth}, board evaluation: ${absoluteEval}cp`
    stopUpdatingResultText()
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



// UPDATE THE CURRENT RESULT TEXT

let updatingResultText = false
async function startUpdatingResultText() {
  updatingResultText = true
  while (updatingResultText) {
    console.log('.')
    const resultText = readSearchResult()
    engineCurrentResult.innerHTML = stringifySearchResult(resultText)
    await new Promise(resolve => setTimeout(resolve, 100))
  }
}
function stopUpdatingResultText() {
  updatingResultText = false
}

function stringifySearchResult(searchResult) {
  const from = searchResult.moveInfo.from
  const to = searchResult.moveInfo.to
  const promotion = searchResult.moveInfo.promotion
  const fromStr = `${String.fromCharCode(from[0] + 97)}${from[1] + 1}`
  const toStr = `${String.fromCharCode(to[0] + 97)}${to[1] + 1}`
  const promotionStr = promotion ? `=${promotion}` : ''
  return `[depth ${searchResult.depth}] ${fromStr}${toStr}${promotionStr}`
}

// See src/searcher/search_result.rs for the memory layout
function readSearchResult() {
  const moveInfo = readMoveInfo()
  const evaluation = readI32(scorePtr)
  const depth = readU8(depthPtr)
  return {moveInfo, evaluation, depth}
}
// See src/types/move_info.rs for the memory layout
function readMoveInfo() {
  const fromX = protochessMemory[mvFromPtr]
  const fromY = protochessMemory[mvFromPtr + 1]
  const toX = protochessMemory[mvToPtr]
  const toY = protochessMemory[mvToPtr + 1]
  // Rust uses 0..MAX_UNICODE for Some(char) and MAX_UNICODE + 1 for None
  const MAX_UNICODE = 0x10FFFF
  const promo = readI32(mvPromoPtr)
  const hasPromo = promo <= MAX_UNICODE
  // Important: use String.fromCodePoint instead of String.fromCharCode
  // because promotions can be any Unicode character
  const promotion = hasPromo ? String.fromCodePoint(promo) : undefined
  return {
    from: [fromX, fromY],
    to: [toX, toY],
    promotion,
  }
}
// Little-endian 32-bit integer
function readI32(ptr) {
  return protochessMemory[ptr] +
    (protochessMemory[ptr + 1] << 8) +
    (protochessMemory[ptr + 2] << 16) +
    (protochessMemory[ptr + 3] << 24)
}
// 8-bit unsigned integer
function readU8(ptr) {
  return protochessMemory[ptr]
}
