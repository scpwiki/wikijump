function isLocaleAliasSrc(prop, bundle){
	if(!bundle){ return false; }
	var isAlias = false;
	var LOCALE_ALIAS_MARK = '@localeAlias';

	for(x in bundle){
		if(x.indexOf(LOCALE_ALIAS_MARK) > 0){
			var prefix = x.substring(0,x.indexOf(LOCALE_ALIAS_MARK));
			if(prop.indexOf(prefix) == 0){
				isAlias = true;
			}
		}
	}	
	return isAlias;
}

function getNativeBundle(filePath){
	//summary:get native bundle content with utf-8 encoding
	//	      native means the content of this bundle is not flatten with parent 
	try{
		var content = fileUtil.readFile(filePath, "utf-8");
		return (!content || !content.length) ? {} : dojo.fromJson(content);
	}catch(e){return{};}
}

function compare(a/*String or Array*/, b/*String or Array*/){
	//summary: simple comparison
	if(dojo.isArray(a) && dojo.isArray(b)){
		for(var i = 0; i < a.length; i++){
			if(a[i] != b[i]){ return false; }
		}
		return true;
	}
	return a==b;
}
