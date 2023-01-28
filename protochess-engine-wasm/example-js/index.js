/*
 * Copyright 2022 Google Inc. All Rights Reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *     http://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
  protochess.set_num_threads(4)
  initUI()
}

async function initUI() {
  updateBoard('Ok')
  manualMoveButton.onclick = manualMoveButtonClick
  engineMoveButton.onclick = engineMoveButtonClick
  fenButton.onclick = fenButtonClick
}

async function updateBoard(result, winner) {
  boardDisplay.innerHTML = await protochess.to_string()
  if (result !== 'Ok') {
    let winnerString;
    if (winner === 0) {
      winnerString = 'White wins'
    } else if (winner === 1) {
      winnerString = 'Black wins'
    } else {
      winnerString = 'Draw'
    }
    boardStatus.innerHTML = result + ', ' + winnerString
  } else if (await protochess.to_move_in_check()) {
    boardStatus.innerHTML = 'Check!'
  } else {
    boardStatus.innerHTML = ' '
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
    const {result, winner_player} = await protochess.make_move_str(manualMoveInput.value)
    if (result === 'IllegalMove') {
      throw 'This move is illegal'
    }
    updateBoard(result, winner_player)
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
    const {make_move_result, depth} = await protochess.play_best_move_timeout(timeout)
    const {result, winner_player} = make_move_result
    await updateBoard(result, winner_player)
    engineDepth.innerHTML = 'Done! Search depth: ' + depth
  } catch (e) {
    engineMoveError.innerHTML = e
  }
}

async function fenButtonClick() {
  clearErrors()
  try {
    // Attempt to set the FEN
    await protochess.load_fen(fenInput.value)
    updateBoard('Ok')
  } catch (e) {
    fenError.innerHTML = e
  }
}
