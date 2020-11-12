
const { app, BrowserWindow } = require('electron')
const passthru_commander = require('./app/passthru')
const path = require('path');
const url = require('url');

const passthru_lib = require('./index.node');

let mainWin = null;
function createSelectorWindow() {
  mainWin = new BrowserWindow({
    //resizable: false,
    width: 500,
    height: 250,
    webPreferences: {
      nodeIntegration: true,
      enableRemoteModule: true,
      //devTools: false
    }
  })

  mainWin.webContents.openDevTools();
  mainWin.setMenuBarVisibility(false);
  mainWin.loadFile('./app/renderer/selector.html');
}

app.whenReady().then(createSelectorWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createSelectorWindow()
  }
})

app.on('open', () => {
  createMainWin();
});
