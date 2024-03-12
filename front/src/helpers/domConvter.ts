export const replaceImg = (text: string) => {
  const parser = new DOMParser();
  const htmlDoc = parser.parseFromString(text, "text/html");
  console.log("input, ", text);

  const images = htmlDoc.querySelectorAll("img");

  images.forEach((img) => {
    const currentSrc = img.src;
    const encode = (str: string): string => btoa(unescape(encodeURIComponent(str))).replace("/", "_");
    img.src = window.location.protocol + "//" + window.location.host + "/" + "images/" + encode(currentSrc);
  });

  console.log("output, ", htmlDoc.documentElement);

  return htmlDoc.documentElement.innerHTML;
};
