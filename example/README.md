我希望我的这个项目野心大一点。一套代码运行所有平台。

视图层：直接编译到各个平台的原生控件（windows=winui，macos/ios=swift ui，linux=gtk，android=jetpack compose，web=DOM，wechat=WXML）。需要修改编译器架构，抽象出一个统一的Codegen接口，然后由不同的crate实现。
逻辑层与数据层：统一使用JS（非web/wechat使用github.com/meta/hermes）
构建：充分利用Vite的API，`WEFT_PLATFORM=xxx vite build`直接输出`apps/xxx/...`一个项目模板，然后调用宿主机的构建工具（如CMake、Gradle等）。

跨平台接口：统一封装到@weft-ui/platform内部，比如`import { fetch } from "@weft-ui/platform";`。不支持的API在运行时报错。
