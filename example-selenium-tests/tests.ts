import { Builder, Capabilities } from "selenium-webdriver"
import { writeFileSync } from "fs";

async function runTest(){
    let driver = await new Builder().forBrowser('chrome').build(); 
    let num = Math.floor(Math.random() * 100)
    await driver.get("https://google.com/search?q=newtest" + num);
    let title = await driver.getTitle();
    console.log("Ran test I guess " + title);

   
//	await new Promise(resolve => setTimeout(resolve, 3000));

    let file = await driver.takeScreenshot();
    writeFileSync(`./ss/ss-${num}.png`, Buffer.from(file, 'base64'));
    await driver.quit();
}


console.log("Don't forget to set SELENIUM_REMOTE_URL, it is currently set to: " + process.env['SELENIUM_REMOTE_URL']);

for(let i = 0; i < 50; i++){
    runTest();
}
