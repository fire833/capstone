"use strict";
exports.__esModule = true;
var net_1 = require("net");
var server = net_1.createServer(function (socket) {
    socket.on('data', function (data) {
        if (socket.writable) {
            socket.write(data);
        }
    });
    socket.on('error', function (e) {
        console.error(e);
    });
    socket.on('close', function () {
        socket.destroy();
    });
});
server.listen(6661, "0.0.0.0");
