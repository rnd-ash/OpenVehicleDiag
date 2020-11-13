const ipc = require('electron').ipcMain;
const { app, BrowserWindow } = require('electron')
const passthru_commander = require('./app/passthru')
const path = require('path');
const url = require('url');

const passthru_lib = require('./index.node');

let win = null;
function createWindow() {
  win = new BrowserWindow({
    //resizable: false,
    width: 500,
    height: 250,
    webPreferences: {
      nodeIntegration: true,
      enableRemoteModule: true,
      //devTools: false
    }
  })

  win.webContents.openDevTools();
  win.setMenuBarVisibility(false);
  win.loadFile('./app/renderer/selector.html');
}

app.whenReady().then(createWindow);

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

ipc.on('newWindow', (event, file, width, height) => {
  win.setSize(width, height);
  win.center();
  win.loadFile(file);
});
