import { Builder, Capabilities } from "selenium-webdriver"
import { writeFileSync } from "fs";

async function runTest(i: number){
	let browser = (() => {	
		let rand = Math.random();
		if (rand <= 0.3) return "chrome";
		if (rand <= 0.6) return "MicrosoftEdge";
		else return "firefox";
	})();
    let driver = await new Builder().forBrowser(browser).build(); 
    await driver.get("https://google.com/search?q=newtest" + i);

    let title = await driver.getTitle();
    console.log("Ran test I guess " + title);

//	await new Promise(resolve => setTimeout(resolve, 3000));

//    let file = await driver.takeScreenshot();
//	writeFileSync(`./ss/ss-${i}.png`, Buffer.from(file, 'base64'));
    await driver.quit();
}


console.log("Don't forget to set SELENIUM_REMOTE_URL, it is currently set to: " + process.env['SELENIUM_REMOTE_URL']);

for(let i = 0; i < 100; i++){
    runTest(i).then(res => {
        console.log("completed test");
    }).catch(err => {
        console.error(`Error in test ${i}:`, err);
    });
}
