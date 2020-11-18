{$page->setLayout('IframeOverlay')}


{module name="login/LoginModule3" backUrl=$url reset=$reset}



<script type="text/javascript">

	Wikijump.vars.rsakey = "{$key}";
	Wikijump.vars.loginSeed = "{$seed}";
	Wikijump.vars.backUrl = "{$url}";

	{literal}
	function createCookie(name,value,days) {
		if (days) {
			var date = new Date();
			date.setTime(date.getTime()+(days*24*60*60*1000));
			var expires = "; expires="+date.toGMTString();
		}
		else var expires = "";
		document.cookie = name+"="+value+expires+"; path=/";
	}
	{/literal}


</script>
