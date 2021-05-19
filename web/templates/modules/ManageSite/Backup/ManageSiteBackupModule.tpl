<h1>{t}Site backup{/t}</h1>

<p>
	{t}This option allows you to create and download a snapshot of this site.
	Currently we are just testing this option and will extend it soon but even though it might be
	useful when you want to make sure your content is safe and stored in multiple places, including your
	own computer.{/t}
</p>
<p>
	{t}The backup option at the moment has a few limitations: you cannot restore from it automatically,
	it does not include all page revisions, only current (latest), it does not include forum discussion or page comments.{/t}
</p>

<h2>{t}Current backup status{/t}</h2>

{if isset($backup)}
	<table style="margin: 0 auto;">
		{if $backup->getStatus()==""}
			<tr>
				<td>{t}Status{/t}:</td>
				<td>{t}queued for processing...{/t}</td>
			</tr>
			<tr>
				<td>{t}Date requested{/t}:</td>
				<td><span class="odate">{$backup->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span></td>
			</tr>
		{/if}
		{if $backup->getStatus()=="started"}
			<tr>
				<td>{t}Status{/t}:</td>
				<td>{t}processing...{/t}</td>
			</tr>
			<tr>
				<td>{t}Date requested{/t}:</td>
				<td><span class="odate">{$backup->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span></td>
			</tr>
		{/if}
		{if $backup->getStatus()=="failed"}
			<tr>
				<td>{t}Status{/t}:</td>
				<td>{t}backup failed for some reason{/t}</td>
			</tr>
			<tr>
				<td>{t}Date requested{/t}:</td>
				<td><span class="odate">{$backup->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span></td>
			</tr>
		{/if}
		{if $backup->getStatus()=="completed"}
			<tr>
				<td>{t}Status{/t}:</td>
				<td><strong>{t}completed{/t}</strong></td>
			</tr>
			<tr>
				<td>{t}Date completed{/t}:</td>
				<td><span class="odate">{$backup->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span></td>
			</tr>
			<tr>
				<td>{t}Backup file size{/t}:</td>
				<td>{$size|escape}</td>
			</tr>
			<tr>
				<td colspan="2">
					<p style="text-align: center;">
						<a href="/local--backup/{$backup->getRand()}/backup.zip">download the ZIPped backup</a>
						| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteBackupModule.listeners.deleteBackup(event)">delete backup</a>
					</p>
				</td>
			</tr>
		{/if}
	</table>

	{if $backup && ($backup->getStatus() == "" || $backup->getStatus() == "started")}
		<p style="text-align: center;">
			<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-backup');">{t}refresh status{/t}</a>
		</p>
		{if $backup->getStatus() == ""}
			<p style="text-align: center;">
				({t}The queue is processed every minute.{/t})
			</p>
		{/if}
	{/if}


{else}
	<p>
		{t}No backup is available for download.{/t}
	</p>
{/if}
{if !$backup || $backup->getStatus()=="completed" || $backup->getStatus()=="failed"}
	<p>
		<a href="javascript:;"  onclick="$('create-backup-form').style.display='block'" >{t}create a new backup{/t}</a>
	</p>
{/if}
<div id="create-backup-form" style="display: none;">
	<h2>{t}Create a backup{/t}</h2>

	<form id="backup-form">

		<table class="form">
			<tr>
				<td>
					{t}Include page sources{/t}
				</td>
				<td>
					<input name="backupSources" type="checkbox" class="checkbox" checked="checked"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}Include file attachments{/t}
				</td>
				<td>
					<input name="backupFiles" type="checkbox" class="checkbox" checked="checked"/>
				</td>
			</tr>
		</table>

		<div class="buttons">
			<input type="button" onclick="$('create-backup-form').style.display='none'" value="{t}cancel{/t}"/>
			<input type="button" value="{t}create backup{/t}" onclick="Wikijump.modules.ManageSiteBackupModule.listeners.requestBackup(event)"/>
		</div>
	</form>

</div>
