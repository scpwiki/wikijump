<?php /* Smarty version 2.6.7, created on 2009-01-04 19:49:30
         compiled from /var/www/wikidot/templates/modules/changes/SiteChangesListModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'pager', '/var/www/wikidot/templates/modules/changes/SiteChangesListModule.tpl', 3, false),array('function', 'printuser', '/var/www/wikidot/templates/modules/changes/SiteChangesListModule.tpl', 43, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/changes/SiteChangesListModule.tpl', 14, false),array('block', 't', '/var/www/wikidot/templates/modules/changes/SiteChangesListModule.tpl', 18, false),)), $this); ?>
<?php if ($this->_tpl_vars['revisions']): ?>

	<?php echo smarty_function_pager(array('jsfunction' => "WIKIDOT.modules.SiteChangesModule.listeners.updateList",'total' => $this->_tpl_vars['pagerData']['totalPages'],'known' => $this->_tpl_vars['pagerData']['knownPages'],'current' => $this->_tpl_vars['pagerData']['currentPage']), $this);?>



	<?php if (count($_from = (array)$this->_tpl_vars['revisions'])):
    foreach ($_from as $this->_tpl_vars['revision']):
?>

		<?php $this->assign('page', $this->_tpl_vars['revision']->getPage()); ?>
		<div class="changes-list-item">

			<table>
				<tr>
					<td class="title">
						<a href="/<?php echo $this->_tpl_vars['page']->getUnixname(); ?>
"><?php if (((is_array($_tmp=$this->_tpl_vars['page']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp))):  echo ((is_array($_tmp=$this->_tpl_vars['page']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  else:  echo ((is_array($_tmp=$this->_tpl_vars['page']->getUnixName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  endif; ?></a>
					</td>
					<td class="flags">
						<?php if ($this->_tpl_vars['revision']->getFlagNew()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>new page created<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">N</span>
					 	<?php endif; ?>
					 	<?php if ($this->_tpl_vars['revision']->getFlagText()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>content source text changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">S</span>
					 	<?php endif; ?>
					 	<?php if ($this->_tpl_vars['revision']->getFlagTitle()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>title changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">T</span>
					 	<?php endif; ?>
					 	<?php if ($this->_tpl_vars['revision']->getFlagRename()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>page renamed/moved<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">R</span>
					 	<?php endif; ?>
					 	<?php if ($this->_tpl_vars['revision']->getFlagFile()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>file/attachment action<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">F</span>
					 	<?php endif; ?>
					 	<?php if ($this->_tpl_vars['revision']->getFlagMeta()): ?>
					 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>meta data changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">M</span>
					 	<?php endif; ?>
					</td>
					<td  class="mod-date">
						<span class="odate"><?php echo $this->_tpl_vars['revision']->getDateLastEdited()->getTimestamp(); ?>
|%e %b %Y - %H:%M:%S|agohover</span>
					</td>
					<td class="revision-no">
						(<?php if ($this->_tpl_vars['revision']->getRevisionNumber() == 0):  $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>new<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack);  else:  $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>rev<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>. <?php echo $this->_tpl_vars['revision']->getRevisionNumber();  endif; ?>)
					</td>
					<td class="mod-by">
						<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['revision']->getUserOrString()), $this);?>

					</td>
				</tr>
			</table>

			<?php if ($this->_tpl_vars['revision']->getComments()): ?>
				<div class="comments">
					<?php echo $this->_tpl_vars['revision']->getComments(); ?>

				</div>
			<?php endif; ?>

					</div>
	<?php endforeach; endif; unset($_from); ?>

	<?php if ($this->_tpl_vars['revisionsCount'] > 10): ?>
		<?php echo smarty_function_pager(array('jsfunction' => "WIKIDOT.modules.SiteChangesModule.listeners.updateList",'total' => $this->_tpl_vars['pagerData']['totalPages'],'known' => $this->_tpl_vars['pagerData']['knownPages'],'current' => $this->_tpl_vars['pagerData']['currentPage']), $this);?>

	<?php endif;  else: ?>
	<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sorry, no revisions matching your criteria.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack);  endif; ?>