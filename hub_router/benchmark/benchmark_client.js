"use strict";
exports.__esModule = true;
var net_1 = require("net");
var crypto_1 = require("crypto");
var TEST_LENGTH_SECONDS = 5;
function run_benchmark(dst_port, dst_ip, completion_callback) {
    var client = new net_1.Socket();
    var start_time = Date.now();
    var bytes_flown = 0;
    var dummy_data = crypto_1.randomBytes(Math.pow(2, 15));
    client.connect(dst_port, dst_ip, function () {
        console.log("Connected to server");
        client.write(dummy_data);
    });
    client.on('data', function (data) {
        bytes_flown += data.length;
        client.write(dummy_data);
        if (Date.now() - start_time > TEST_LENGTH_SECONDS * 1000) {
            console.log("Transmitted " + bytes_flown + " bytes in 1sec");
            client.end();
            client.destroy();
            completion_callback(bytes_flown);
        }
    });
    client.on('close', function () {
        console.log("closed");
    });
}
var direct_connection = { port: 6661, ip: "127.0.0.1" };
var proxied_connection = { port: 6543, ip: "127.0.0.1" };
var direct_results = [];
var proxied_results = [];
for (var i = 0; i < 200; i++) {
    setTimeout(function () {
        run_benchmark(direct_connection.port, direct_connection.ip, function (num_sent) {
            direct_results.push(num_sent);
        });
    }, TEST_LENGTH_SECONDS * 1000 * 1.5);
    setTimeout(function () {
        run_benchmark(proxied_connection.port, proxied_connection.ip, function (num_sent) {
            proxied_results.push(num_sent);
        });
    }, 1);
}
setTimeout(function () {
    console.log(direct_results);
    console.log(proxied_results);
    console.log(" Direct mean: " + direct_results.reduce(function (acc, el) { return acc + el; }) / direct_results.length);
    console.log("Proxied mean: " + proxied_results.reduce(function (acc, el) { return acc + el; }) / proxied_results.length);
}, TEST_LENGTH_SECONDS * 3 * 1000);
