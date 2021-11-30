<div class="owindow error" style="width: 80%;">
	<div class="title">
		{t}Lock conflict{/t}
	</div>
	<div class="content">
		<h1>{t}Unable to recreate lock safely{/t}</h1>
		<p>
			{t}The page is currently edited by another user in a way that it is not possible
			to safely recreate lock.{/t}
		</p>
		<p>
			{t}It is recommended that you cancel your edit session and wait until the lock is released.
			However if you are sure the imposed lock(s) can be safely deleted you
			can forcibly recreate your lock by deleting conflicting locks.{/t}
		</p>
		{foreach from=$locks item=lock}
		<p>
			{t}Locked by{/t}: {printuser user=$lock->getUserOrString() image="true"}<br/>
			{t}Started editing{/t}: <span class="odate">{$lock->getDateStarted()->getTimestamp()}</span> ({$lock->getStartedAgo()} {t}seconds ago{/t})<br/>
			{t}Lock will expire in{/t}: {$lock->getExpireIn()} {t}seconds (if user remains inactive){/t}</br>
		</p>
		{/foreach}
		<div class="button-bar">
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
			<a href="javascript:;" onclick="Wikijump.modules.PageEditModule.listeners.forceLockIntercept(event)">{t}forcibly recreate lock{/t}</a>
		</div>
	</div>
</div>
