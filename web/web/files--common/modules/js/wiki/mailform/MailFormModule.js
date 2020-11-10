

Wikijump.modules.MailFormModule = {};

Wikijump.modules.MailFormModule.vars = {};

Wikijump.modules.MailFormModule.listeners = {
	send: function(e, formRand){
		var form = $('mailform-'+formRand);

		// dump the values
		var p = new Object();
		p.formdata = JSON.stringify(OZONE.utils.formToArray(form));
		p.action = 'wiki/MailFormAction';
		p.event = 'sendForm';
		p.formdef = $("mailformdef-"+formRand).innerHTML;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.MailFormModule.callbacks.send);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Sending the form...";
		w.show();
		Wikijump.modules.MailFormModule.vars.rand = formRand;
	}
}

Wikijump.modules.MailFormModule.callbacks = {
	send: function(r){
		var rand = Wikijump.modules.MailFormModule.vars.rand;
		// remove 'invalid...' class
		var form = $('mailform-'+rand);
		var trs = form.getElementsByTagName('tr');
		for(var i=0; i<trs.length; i++){
			YAHOO.util.Dom.removeClass(trs[i],'invalid-value-row');
		}

		if(r.status == 'form_errors'){
			var errors = r.errors;

			for(var n in errors){
				var row = $('mailform-row-'+rand+'-'+n);
				YAHOO.util.Dom.addClass(row,'invalid-value-row');
				var errorDiv = YAHOO.util.Dom.getElementsByClassName('field-error-message', 'div', row)[0];
				errorDiv.innerHTML = errors[n];
			}
			OZONE.dialog.cleanAll();
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The form has been sent. Thank you!";
		w.show();

		var sp = r.successPage;
		if(sp){
			setTimeout("window.location.href='/"+sp+"'", 500);
		}else{
			form.innerHTML = "<strong>The form has been sent. Thank you!</strong>";
		}
	}

}
