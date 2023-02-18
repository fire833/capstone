import {Socket} from "net"
import {randomBytes} from "crypto"



const TEST_LENGTH_SECONDS = 5;

function run_benchmark(dst_port: number, dst_ip: string, completion_callback: (num: number) => void){
    let client = new Socket();

    let start_time = Date.now();

    let bytes_flown = 0;

    let dummy_data: Buffer = randomBytes(2**15);

    client.connect(dst_port, dst_ip, () => {
        console.log("Connected to server");
        client.write(dummy_data);
    });
    
    client.on('data', data => {
        bytes_flown += data.length;
        client.write(dummy_data);

        if(Date.now() - start_time > TEST_LENGTH_SECONDS * 1000){
            console.log(`Transmitted ${bytes_flown} bytes in 1sec`);
            client.end();
            client.destroy();
            completion_callback(bytes_flown);
        }
    })
    
    client.on('close', () => {
        console.log("closed");
    })

}





let direct_connection = {port: 6661, ip: "127.0.0.1"};
let proxied_connection = {port: 6543, ip: "127.0.0.1"};

let direct_results: number[] = [];
let proxied_results: number[] = [];



for(let i = 0; i < 200; i++){
    setTimeout(() => {
        run_benchmark(direct_connection.port, direct_connection.ip, num_sent => {
            direct_results.push(num_sent)
        } )
    }, TEST_LENGTH_SECONDS * 1000 * 1.5);


    setTimeout(() => {
        run_benchmark(proxied_connection.port, proxied_connection.ip, num_sent => {
            proxied_results.push(num_sent)
        } )
    }, 1);
}

setTimeout(() => {
    console.log(direct_results);
    console.log(proxied_results);
    console.log(" Direct mean: " + direct_results.reduce((acc, el) => acc + el) / direct_results.length);
    console.log("Proxied mean: " + proxied_results.reduce((acc, el) => acc + el) / proxied_results.length);          
}, TEST_LENGTH_SECONDS * 3 * 1000)

