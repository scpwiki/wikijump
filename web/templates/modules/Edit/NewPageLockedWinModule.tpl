
<div class="owindow" style="width: 80%;">
	<div class="title">
		{t}Page locked{/t}
	</div>
	<div class="content">
		<h1>{t}The page you want to create is locked...{/t}</h1>
		<p>
			{t}... which means somebody else is creating the page at the same time
			and owns an exclusive page lock.{/t}
		</p>
		<p>
			{t}If however you are sure you want to intercept the page lock or the lock comes
			from a mistake or failure, you may try to forcibly remove conflicting lock.{/t}
		</p>
		{foreach from=$locks item=lock}
		<p>
			{t}By{/t}: {printuser user=$lock->getUserOrString() image="true"}<br/>
			{t}Started editing{/t}: <span class="odate">{$lock->getDateStarted()->getTimestamp()}</span> ({$lock->getStartedAgo()} {t}seconds ago{/t})<br/>
			{t}Lock will expire in{/t}: {$lock->getExpireIn()} {t}seconds (if user remains inactive){/t}</br>
		</p>
		{/foreach}
		<div class="button-bar">
			<a href="javascript:;" onclick="window.location.reload()">{t}cancel{/t}</a>
			<a href="javascript:;"  onclick="Wikijump.modules.PageEditModule.listeners.forcePageEditLockRemove(event)">{t}try to intercept the lock{/t}</a>,
		</div>
	</div>
</div>
