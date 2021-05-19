<div class="title">Abuse report</div>
<div class="content">
	<h1>User abuse report</h1>

	<p>
		If you think this user violates Terms of Service of {$SERVICE_NAME},
		posts objectionable content etc., please send this report.
	</p>
	<p>
		The report will be sent to administrators of {$SERVICE_NAME} who will
		act accordingly.
		{if isset($site)}
			If you think the problem
			also concerns this site administrators/moderators and can be resolved locally,
			please check the option below.
		{/if}
	</p>
	<form>
		User name: <strong>{$user->username|escape}</strong><br/>
		<br/>
		Problem description:<br/>
		<textarea name="text" id="abuse-report-text" cols="30" rows="5" style="width: 95%"></textarea>
		<div>(<span id="abuse-report-chcount"></span> characters left)</div>
		{if isset($site)}
			<br/>
			<input class="checkbox" type="checkbox" checked="checked"> Send to <em>{$site->getName()|escape}</em> administrators too
		{/if}
	</form>
</div>
<div class="button-bar">
	<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">cancel</a>
	<a href="javascript:;" onclick="Wikijump.modules.UserAbuseReportModule.listeners.sendReport(event)">send report</a>
</div>
