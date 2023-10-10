import * as esbuild from "esbuild";
import { getLogger } from "./log";
const log = getLogger("");
import { ESLint } from "eslint";

/**
 * Function to update lines when something happens
 * @param input The text you want to print
 * @param _isBuiltInput Whether you are printing `Built in x ms` or not
 */
const updateLine = (input: string, _isBuiltInput: boolean = false) => {
  log.debug(input);
};

const eslint = new ESLint();
/**
 * Builds the code in no time
 */
export const build_code = async () => {
  try {
    // Get time before build starts
    const timerStart = Date.now();

    const results = await eslint.lintFiles([
      "./client/**/*.ts",
      "./plugins/**/*.ts",
    ]);

    const formatter = await eslint.loadFormatter("stylish");

    let errorCount = 0;
    let warnCount = 0;
    for (const result of results) {
      errorCount += result.errorCount + result.fatalErrorCount;
      warnCount += result.warningCount;
      if (result.errorCount + result.warningCount > 0) {
        console.log(formatter.format([result]));
      }
    }

    if (errorCount + warnCount > 0) {
      return;
    }

    // Build code
    await esbuild.build({
      color: true,
      entryPoints: ["./client/index.ts"],
      outfile: "./build/medium.js",
      bundle: true,
      sourcemap: true,
      tsconfig: "./tsconfig.json",
      logLevel: "error",
      define: {
        "process.env.NODE_ENV": "'dev'",
      },
    });

    for (const name of ["keyboard-input", "mouse-input", "controller-input"]) {
      await build_plugin(name);
    }

    // Get time after build ends
    const timerEnd = Date.now();
    updateLine(`Code built in ${timerEnd - timerStart}ms.`, true);
  } catch (e) {
    log.error(e);
    // OOPS! ERROR!
  }
};

const build_plugin = async (plugin_file_name: string) => {
  await esbuild.build({
    color: true,
    entryPoints: [`./plugins/${plugin_file_name}.ts`],
    outfile: `./build/${plugin_file_name}.js`,
    bundle: true,
    sourcemap: true,
    tsconfig: "./tsconfig.json",
    logLevel: "error",
    define: {
      "process.env.NODE_ENV": "'dev'",
    },
  });
};
