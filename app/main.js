const ipc = require('electron').ipcMain;
const { app, BrowserWindow } = require('electron')
const path = require('path');
const url = require('url');
let consts = require('./app/ptconsts');
let passthru = require('./app/passthru');
let fs = require('./app/fs');

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
    createWindow()
  }
})

app.on('open', () => {
  createWindow();
});

ipc.on('newWindow', (event, file, width, height) => {
  win.setSize(width, height);
  win.center();
  win.loadFile(file);
});

/*
ipc.on("f_open",(event) => {
  console.log("Opening file chooser");
  const {dialog} = require('electron');
  const file = dialog.showOpenDialog(win, {
    properties: ['openFile'],
    filters: [{ name: 'OVD ECU Json', extensions: ['ovdjson'] }]
  });
  console.log(file);
});
*/