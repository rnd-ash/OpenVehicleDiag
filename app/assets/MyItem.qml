import QtQuick 2.0
import QtQuick.Controls 2.2
import QtQuick.Layouts 1.3

RowLayout {
    property string name
    CheckBox {
        text: "Checkbox for " + name
    }
    Button {
        text: "Hi!"
    }
}