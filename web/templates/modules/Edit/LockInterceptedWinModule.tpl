<div class="owindow" style="width: 80%;">
	<div class="title">
		{t}Lock intercepted{/t}
	</div>
	<div class="content">
		<h1>{t}The edit lock has been (forcibly) removed{/t}</h1>
		<p>
			{t}While you have been editing the page contents someone else removed your
			edit lock and started editing the page in a way that your actions conflict.{/t}
		</p>
		{if $nonrecoverable}
			<p>
				{t}Moreover the page content has been already changed. The safest solution is
				to stop editing, review changes made to the page and apply your changes again.{/t}
			</p>
			<p>
				{t}It is recommended that you now see the "diff" changes of your current edit,
				wait until the other lock expires or is released and edit the page source
				and insert your changes again.{/t}
			</p>

			<div class="button-bar">
				<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close this window{/t}</a>
			</div>

		{else}
			<p>
				{t}The conflicting locks are{/t}:
			</p>
			{foreach from=$locks item=lock}
				<p>
					{t}By{/t}: {printuser user=$lock->getUserOrString() image="true"}<br/>
					{t}Started editing{/t}: <span class="odate">{$lock->getDateStarted()->getTimestamp()}</span> ({$lock->getStartedAgo()} {t}seconds ago{/t})<br/>
					{t}Lock will expire in{/t}: {$lock->getExpireIn()} {t}seconds (if user remains inactive){/t}</br>
				</p>
			{/foreach}
			<p>
				{t escape=no}By choosing "force lock recreate" you will remove conflicting locks and try to recreate
				your lock. <strong>Use with caution.</strong>{/t}
			</p>
			<p>
				{t}Warning: please note that page lock interception should not be used without a reason.
				While our mechanisms guarantee gracefull exits from conflicts without uncontrolled
				content loss, please use lock interception only when necessary.{/t}
			</p>
			<div class="button-bar">
				<a href="javascript:;" onclick="Wikijump.modules.PageEditModule.listeners.forceLockIntercept(event)">{t}force lock aquire{/t}</a>
				<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close this window{/t}</a>
			</div>

		{/if}
	</div>
</div>
