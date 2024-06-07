import"./MarkdownEditor-1a13690f1612bf18.js";window.SunIcon=document.getElementById("theme-icon-sun");window.MoonIcon=document.getElementById("theme-icon-moon");window.toggle_theme=()=>{if(window.PASTE_USES_CUSTOM_THEME&&window.localStorage.getItem("bundles:user.ForceClientTheme")!=="true")return;if(window.localStorage.getItem("theme")==="dark")document.documentElement.classList.remove("dark-theme"),window.localStorage.setItem("theme","light"),window.SunIcon.style.display="flex",window.MoonIcon.style.display="none";else document.documentElement.classList.add("dark-theme"),window.localStorage.setItem("theme","dark"),window.SunIcon.style.display="none",window.MoonIcon.style.display="flex"};if(window.matchMedia("(prefers-color-scheme: dark)").matches&&!window.localStorage.getItem("theme"))document.documentElement.classList.add("dark-theme"),window.localStorage.setItem("theme","dark"),window.SunIcon.style.display="none",window.MoonIcon.style.display="flex";else if(window.matchMedia("(prefers-color-scheme: light)").matches&&!window.localStorage.getItem("theme"))document.documentElement.classList.remove("dark-theme"),window.localStorage.setItem("theme","light"),window.SunIcon.style.display="flex",window.MoonIcon.style.display="none";else if(window.localStorage.getItem("theme")){const j=window.localStorage.getItem("theme");if(document.documentElement.className=`${j}-theme`,j.includes("dark"))window.SunIcon.style.display="none",window.MoonIcon.style.display="flex";else window.SunIcon.style.display="flex",window.MoonIcon.style.display="none"}if(!window.PASTE_USES_CUSTOM_THEME||window.localStorage.getItem("bundles:user.ForceClientTheme")==="true"){const j=document.createElement("style");j.innerHTML=window.localStorage.getItem("bundles:user.GlobalCSSString"),document.body.appendChild(j)}setTimeout(()=>{for(let j of Array.from(document.querySelectorAll(".date-time-to-localize")))j.innerText=new Date(parseInt(j.innerText)).toLocaleString()},50);for(let j of Array.from(document.querySelectorAll('[disabled="false"]')))j.removeAttribute("disabled");setTimeout(()=>{for(let j of Array.from(document.querySelectorAll("a[disabled]")))j.removeAttribute("href")},50);var F=document.querySelectorAll(".dismissable");for(let j of Array.from(F))if(window.sessionStorage.getItem(`dismissed:${j.id}`)==="true")j.remove();else{const A=j.querySelector(".dismiss");if(A)A.addEventListener("click",()=>{window.sessionStorage.setItem(`dismissed:${j.id}`,"true"),j.remove()})}var E=document.querySelectorAll("h1, h2, h3, h4, h5, h6");for(let j of Array.from(E)){j.style.cursor="pointer",j.title=j.innerText;const q=j.querySelector("a.anchor");if(q)j.id=q.id,q.removeAttribute("id"),q.remove();else j.id=encodeURIComponent(j.innerText.toLowerCase());if(window.location.hash===`#${j.id}`)j.style.background="var(--yellow1)",j.scrollTo();j.addEventListener("click",()=>{window.location.hash=j.id,window.navigator.clipboard.writeText(window.location.href);for(let A of Array.from(E))A.style.background="unset";j.style.background="var(--yellow1)",j.scrollTo()})}var G=document.querySelectorAll(".avatar");for(let j of Array.from(G))if(j.complete){if(j.naturalWidth!==0)continue;j.remove()}else j.addEventListener("error",()=>{j.remove()});var H=Array.from(document.querySelectorAll("[b_onclick]"));for(let j of H)j.setAttribute("onclick",j.getAttribute("b_onclick")),j.removeAttribute("b_onclick");globalThis.toggle_child_menu=(j,q,A=!0)=>{while(j.nodeName!=="BUTTON")j=j.parentElement;const z=document.querySelector(q);if(z){if(j.classList.toggle("selected"),z.style.display==="none"){let D=j.getBoundingClientRect();if(A===!0)z.style.top=`${D.bottom+j.offsetTop}px`;else z.style.bottom=`${D.top+j.offsetTop}px`;z.style.display="flex",j.style.background="var(--background-surface)",j.style.filter="invert(1) grayscale(1)",z.addEventListener("click",(B)=>{B.stopPropagation()}),setTimeout(()=>{let B=()=>{window.toggle_child_menu(j,q),window.removeEventListener("click",B),j.removeEventListener("click",C)};window.addEventListener("click",B);let C=()=>{window.toggle_child_menu(j,q),j.removeEventListener("click",C)};j.addEventListener("click",C)},100)}else if(z.style.display==="flex")z.style.display="none",j.style.background="inherit",j.style.filter="unset"}};for(let j of Array.from(document.querySelectorAll('[data-wants-redirect="true"]')))j.href=`${j.href}?callback=${encodeURIComponent(`${window.location.origin}/api/auth/callback`)}`;for(let j of Array.from(document.querySelectorAll("[data-dialog]"))){const q=document.getElementById(j.getAttribute("data-dialog"));j.addEventListener("click",()=>{q.showModal()})}window.addEventListener("click",(j)=>{if(j.target.tagName!=="DIALOG")return;const q=j.target.getBoundingClientRect();if((q.top<=j.clientY&&j.clientY<=q.top+q.height&&q.left<=j.clientX&&j.clientX<=q.left+q.width)===!1)j.target.close()});var I={};export{I as default};
