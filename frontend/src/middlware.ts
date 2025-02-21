import { defineMiddleware } from "astro:middleware";
import { checkUser } from "./api/utils";

export const onRequest = defineMiddleware(async (context, next) => {
  return next();
  if (await checkUser()) {
    console.log("User is logged in");
    return next();
  }

  console.log("user is not logged in");

  if (
    context.url.pathname === "/login" ||
    context.url.pathname === "/register"
  ) {
    console.log("User is not logged in and is going to login");
    return next();
  } else {
    console.log("User is not logged in and is being redirected to login");
    return context.rewrite(
      new Request("http://127.0.0.1/login", {
        headers: {
          "x-redirect-to": context.url.pathname,
        },
      })
    );
  }
});
