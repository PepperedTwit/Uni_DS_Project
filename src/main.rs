use neon::prelude::*;
use neon::context::Context;

fn scrape_website(mut cx: FunctionContext) -> JsResult<JsString> {
    
    let url = cx.argument::<JsString>(0)?.value(&mut cx);
    
    // Call the JavaScript scraping function
    let global = cx.global();

    let scraper_module: Handle<JsObject> = global.get(&mut cx, "scraper")?;

    let scrape_fn: Handle<JsFunction> = scraper_module.get(&mut cx, "scrapeWebsite")?;

    let result = scrape_fn.call(&mut cx, global, vec![cx.string(url)])?;
    
    Ok(result.downcast::<JsString>().or_throw(&mut cx)?)

}

register_module!(mut cx, {cx.export_function("scrapeWebsite", scrape_website)});


fn main() {
    let url = "https://www.forbes.com/billionaires/page-data/index/page-data.json";
    println!("Scraping: {}", url);
    
    // Initialize Node.js runtime and call the scrape_website function
    let mut runtime = neon::runtime::Runtime::new().unwrap();
    let result = runtime.run(|cx| {
        let scrape_fn = cx.global().get::<JsFunction, _, _>(cx, "scrapeWebsite").unwrap();
        let args = vec![cx.string(url)];
        scrape_fn.call(cx, cx.undefined(), args)
    }).unwrap();
    
    println!("Result: {:?}", result);
}