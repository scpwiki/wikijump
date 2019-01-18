<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html 
     PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
     "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="pl" lang="pl">

{$page->addStyleSheet("base.css")} 

{$page->addJavaScript("base.js")} 
{$page->addJavaScript("contextmenu.js")} 
{$page->addJavaScript("infobox-photoinfo1.js")}
{$page->addHttpEquiv("content-type", "text/html;charset=UTF-8")}

{$page->addHttpEquiv("content-language", "pl")}

{$page->addMeta("description","Internetowa galeria fotograficzna")}
{$page->addMeta("keywords","zdjęcia, canon, nikon, minolta, aparat cyfrowy, druk cyfrowy, kredyt hipoteczny, fotografia, profesjonalna, amatorska, galerie, galeria,  foto, dyskusje, recenzje, sprzęt, fotograficzny, reklama, agencja reklamowa, wiadomości, aparat, aparaty")}
{$page->addMeta("content","zdjęcia, fotografia, olympus, canon, nikon, profesjonalna, amatorska, galerie, galeria, foto, dyskusje, recenzje, sprzęt, fotograficzny, wiadomości, aparat, aparaty, aparaty cyfrowe, olympus, nikon, canon, kredyt hipoteczny, aparat cyfrowy, reklama, agencja reklamowa, druk cyfrowy")}
 {$page->addLink('shortcut icon', 'http://www.fotoforum.pl/favicon.ico', 'image/x-icon')}

{macro name="OzoneXHTMLHead" page="$page"}
  <body {macro name="OzoneXHTMLBodyProperties" page=$page}>
	{$screen_placeholder}
  </body>
</html>