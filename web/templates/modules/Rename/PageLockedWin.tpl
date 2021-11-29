<div class="owindow" style="width: 80%;">
	<div class="title">
		{t}Page locked{/t}
	</div>
	<div class="content">
		<h1>{t}The page is being edited...{/t}</h1>
		<p>
			{t}It seems that the page is currently being edited by another user(s):{/t}
		</p>

		{foreach from=$locks item=lock}
			<p>
				{t}Locked by{/t}: {printuser user=$lock->getUserOrString() image="true"}<br/>
				{t}Started editing{/t}: <span class="odate">{$lock->getDateStarted()->getTimestamp()}</span> ({$lock->getStartedAgo()} {t}seconds ago{/t})<br/>
				{t}Lock will expire in{/t}: {$lock->getExpireIn()} {t}seconds (if user remains inactive){/t}</br>
			</p>
		{/foreach}
		<p>
			{t escape=no}If you are <strong>absolutely</strong> sure that the lock is not valid or have
			a very good reason to remove it, you can force
			the lock removal manually pressing the button below.{/t}
		</p>
		<div class="button-bar">
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
			<a  href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.rename(event)">{t}try again{/t}</a>
			<a  href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.renameForce(event)">{t}remove lock(s) and proceed{/t}</a>
		</div>
	</div>
</div>
