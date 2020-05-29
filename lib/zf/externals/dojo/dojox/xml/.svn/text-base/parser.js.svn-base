dojo.provide("dojox.xml.parser");


// FIXME: 
//		use dojox.data.dom.createDocument instead? It does the same stuff but
//		is much more robust

dojox.xml.parser.parse = function(/*String*/ str){
	// summary:
	//		returns a new native XML document from the string provided as the
	//		single argument to parse(). Parsing errors throw exceptions.
	if(dojo.isIE){
		var nativeDoc = new ActiveXObject("Microsoft.XMLDOM");
		nativeDoc.async = "false";
		nativeDoc.loadXML(str);
		var pe = nativeDoc.parseError;
		if(pe.errorCode !== 0){
			throw new Error("Line: " + pe.line + "\n" +
			 	"Col: " + pe.linepos + "\n" +
				"Reason: " + pe.reason + "\n" + 
				"Error Code: " + pe.errorCode + "\n" +
				"Source: " + pe.srcText);
		}
		return nativeDoc; // DomDocument
	}else{
		var parser = new DOMParser();
		var	nativeDoc = parser.parseFromString(str, "text/xml");
		var de = nativeDoc.documentElement;
		
		var errorNS = "http://www.mozilla.org/newlayout/xml/parsererror.xml";
		if(de.nodeName == "parsererror" && de.namespaceURI == errorNS){
			var sourceText = de.getElementsByTagNameNS(errorNS, 'sourcetext')[0];
			if(!sourceText){
				sourceText = sourceText.firstChild.data;
			}
        	throw new Error("Error parsing text " + nativeDoc.documentElement.firstChild.data + " \n"  +
        		sourceText);
		}					
		return nativeDoc; // DomDocument
	}
}
