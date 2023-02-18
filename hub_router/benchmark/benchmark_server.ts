import {createServer} from "net"

let server = createServer(function(socket){
    socket.on('data', data => {
        if(socket.writable){
            socket.write(data);
        }
    });

    socket.on('error', (e) => {
        console.error(e)
    });

    socket.on('close', () => {
        socket.destroy()
    })

})
server.listen(6661, "0.0.0.0");