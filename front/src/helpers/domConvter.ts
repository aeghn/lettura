export const replaceImg = (text: string) => {
  const parser = new DOMParser();
  const htmlDoc = parser.parseFromString(text, "text/html");
  console.log("input, ", text);

  const images = htmlDoc.querySelectorAll("img");

  images.forEach((img) => {
    img.src = replaceImgUrl(img.src);
  });

  console.log("output, ", htmlDoc.documentElement);

  return htmlDoc.documentElement.innerHTML;
};

export const replaceImgUrl = (url: string) => {
  const encode = (str: string): string => btoa(unescape(encodeURIComponent(str))).replace("/", "_");
  return window.location.protocol + "//" + window.location.host + "/" + "images/" + encode(url);
};
