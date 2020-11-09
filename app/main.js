
const { app, BrowserWindow } = require('electron')
const path = require('path');
const url = require('url');

function createWindow() {
  const selectorWin = new BrowserWindow({
    //resizable: false,
    width: 500,
    height: 250,
    webPreferences: {
      nodeIntegration: true,
      enableRemoteModule: true,
      //devTools: false
    }
  })

  selectorWin.webContents.on('crashed', () => {
    selectorWin.destroy();
    createWindow();
  });
  selectorWin.webContents.openDevTools();
  selectorWin.setMenuBarVisibility(false);
  selectorWin.loadFile('./app/renderer/selector.html');
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
