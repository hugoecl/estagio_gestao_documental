import { defineMiddleware } from "astro:middleware";
import API_BASE_URL from "@api/base-url";

export const onRequest = defineMiddleware(async (context, next) => {
  if (context.originPathname === "/_image/") {
    return next();
  }
  const response = await fetch(`${API_BASE_URL}/users/check`, {
    method: "POST",
    body: context.originPathname,

    // @ts-ignore we don't need to check if the cookie header is present, it already handler the returning null
    headers: {
      cookie: context.request.headers.get("cookie"),
    },
  });

  const ok = response.ok;

  if (!ok) {
    if (
      context.originPathname === "/iniciar-sessao/" ||
      context.originPathname === "/registo/"
    ) {
      return next();
    }
    return context.redirect("/iniciar-sessao/");
  } else {
    if (
      context.originPathname === "/iniciar-sessao/" ||
      context.originPathname === "/registo/"
    ) {
      return context.redirect("/");
    }
  }

  return next();
});
