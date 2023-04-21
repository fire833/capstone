import { Builder, Capabilities } from "selenium-webdriver"
import { writeFileSync } from "fs";

let browserMap = {};
async function runTest(i: number){
	let browser = (() => {	
		let rand = Math.random();
		if (rand <= 0.3) return "chrome";
		if (rand <= 0.6) return "MicrosoftEdge";
		else return "firefox";
	})();
	browserMap[i] = browser;
    let driver = await new Builder().forBrowser(browser).build(); 
    await driver.get("https://google.com/search?q=newtest" + i);

    let title = await driver.getTitle();
    console.log("Ran test" + title);
    await driver.quit();
}


console.log("Don't forget to set SELENIUM_REMOTE_URL, it is currently set to: " + process.env['SELENIUM_REMOTE_URL']);

let num_success = 0;
let num_fail = 0;
const NUM_TESTS = 1000;
async function enqueue_tests() {
	let promises: Promise<any>[] = [];
	for(let i = 0; i < NUM_TESTS; i++){
		if (i % 100 === 0) {
			await new Promise((resolve, _) => setTimeout(resolve, 6000));
		}
		await new Promise((resolve, _) => setTimeout(resolve, 20));
		promises.push(runTest(i).then(res => {
			console.log("completed test");
			num_success++;
		}).catch(err => {
			console.error(`Error in ${browserMap[i]} test ${i}:`, err);
			num_fail++;
		}));
	}
	return promises;
}
enqueue_tests().then(async (promises) => {
	await Promise.all(promises).then(e => {
		console.log(`Success: ${num_success}, Fail: ${num_fail}, Percent: ${num_success / NUM_TESTS * 100}`);
	});
})
