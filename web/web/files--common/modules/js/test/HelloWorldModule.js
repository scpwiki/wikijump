

Wikijump.modules.HelloWorldModule = {};

Wikijump.modules.HelloWorldModule.listener = {
	click: function(e){
		alert('clicked');
		var p = new Object();
		p.action = "NewSiteAction";
		p.event = "..."
		OZONE.ajax.requestModule("test/HelloWorld2Module", p,Wikijump.modules.HelloWorldModule.callbacks.click );
	}

}

Wikijump.modules.HelloWorldModule.callbacks = {

	click: function(r){
		$("hello-world-2-box").innerHTML = r.body;

	}
}
