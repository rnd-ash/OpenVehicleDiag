
const { app, BrowserWindow } = require('electron')
const path = require('path');
const url = require('url');

function createWindow() {
  var selector = require('./app/renderer/selector.js');
  const win = new BrowserWindow({
    width: 500,
    height: 200,
    webPreferences: {
      nodeIntegration: true,
      enableRemoteModule: true,
      //devTools: false
    }
  })
  win.webContents.openDevTools()
  win.setMenuBarVisibility(false);
  win.loadFile('./app/renderer/selector.html');
  win.webContents.executeJavaScript(selector.onWindowCreated(win));
}

app.whenReady().then(createWindow)

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow()
  }
})
