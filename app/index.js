
const { app, BrowserWindow } = require('electron')
const path = require('path');
const url = require('url');

function createWindow() {
  const win = new BrowserWindow({
    width: 500,
    height: 200,
    webPreferences: {
      nodeIntegration: true,
      //devTools: false
    }
  })
  win.setMenuBarVisibility(false);
  win.loadFile('./index.html')
  win.webContents.openDevTools()
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
