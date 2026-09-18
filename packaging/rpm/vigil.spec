%global vigil_stage %{_sourcedir}

%global __os_install_post %{nil}
%global debug_package %{nil}

Name:           vigil
Version:        %{vigil_version}
Release:        1%{?dist}
Summary:        Host protection agent — watches the host it runs on
License:        Apache-2.0
Vendor:         RSQA
AutoReqProv:    no
Packager:       RSQA <vigil@rsqa.space>

%description
vigil reads the host it is installed on — listening sockets, accounts and
logins, and the files that decide who may enter — compares each reading with the
previous one, and turns the difference into findings.

This package installs four programs: vigild, the daemon systemd runs, vigil,
the terminal console that reads its local socket, vigil-audit-plugin, which
auditd starts to hand this host's program launches over, and
vigil-container-dump, which a timer runs to ask this host's container engines
what they hold.

%prep

%build

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a %{vigil_stage}/. %{buildroot}/

%files
%attr(0755,root,root) /usr/sbin/vigild
%attr(0755,root,root) /usr/bin/vigil
%attr(0755,root,root) /usr/sbin/vigil-audit-plugin
%attr(0755,root,root) /usr/sbin/vigil-container-dump
%dir %attr(0700,root,root) /etc/vigil
%config(noreplace) %attr(0600,root,root) /etc/vigil/vigil.yaml
%dir %attr(0700,root,root) /etc/vigil/collectors
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/containers.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/files.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/firewall.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/launches.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/network.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/persistence.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/processes.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/resources.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/collectors/users.yaml
%config(noreplace) %attr(0600,root,root) /etc/vigil/watch_fs.yaml
%dir %attr(0700,root,root) /etc/vigil/suppressions
%dir %attr(0700,root,root) /etc/vigil/reporters
%config(noreplace) %attr(0644,root,root) /etc/logrotate.d/vigil
%dir %attr(0750,root,root) /etc/audit
%dir %attr(0750,root,root) /etc/audit/rules.d
%dir %attr(0750,root,root) /etc/audit/plugins.d
%config(noreplace) %attr(0640,root,root) /etc/audit/rules.d/vigil-exec.rules
%config(noreplace) %attr(0640,root,root) /etc/audit/plugins.d/vigil.conf
%attr(0644,root,root) /usr/lib/systemd/system/vigild.service
%attr(0644,root,root) /usr/lib/systemd/system/vigil-firewall.service
%attr(0644,root,root) /usr/lib/systemd/system/vigil-firewall.timer
%attr(0644,root,root) /usr/lib/systemd/system/vigil-containers.service
%attr(0644,root,root) /usr/lib/systemd/system/vigil-containers.timer
%attr(0644,root,root) /usr/lib/tmpfiles.d/vigil.conf
%dir %attr(0700,root,root) /var/lib/vigil
%dir %attr(0700,root,root) /var/lib/vigil/firewall
%dir %attr(0700,root,root) /var/lib/vigil/containers
%dir %attr(0700,root,root) /var/log/vigil
%dir %attr(0755,root,root) /usr/share/doc/vigil
%doc %attr(0644,root,root) /usr/share/doc/vigil/README.md
%doc %attr(0644,root,root) /usr/share/doc/vigil/NOTICE
%doc %attr(0644,root,root) /usr/share/doc/vigil/THIRD-PARTY.md
%license %attr(0644,root,root) /usr/share/doc/vigil/copyright

%post
if [ -x /usr/bin/systemd-tmpfiles ]; then
    /usr/bin/systemd-tmpfiles --create /usr/lib/tmpfiles.d/vigil.conf >/dev/null 2>&1 || true
fi
for directory in /run/vigil /var/lib/vigil /var/lib/vigil/firewall /var/lib/vigil/containers /var/log/vigil; do
    [ -d "$directory" ] || mkdir -p "$directory"
    chmod 0700 "$directory"
done

if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
    if [ "$1" -ge 2 ]; then
        systemctl try-restart vigild.service >/dev/null 2>&1 || true
    fi
fi

if [ -d /etc/audit ]; then
cat <<'AUDIT'

vigil watches program launches through auditd, and auditd has to be told twice:

  augenrules --load          loads /etc/audit/rules.d/vigil-exec.rules
  systemctl restart auditd   starts the plugin in /etc/audit/plugins.d/vigil.conf

This package does neither. Until both are done vigil reads /var/log/audit/audit.log instead, a
reading late and exposed to rotation, and its own health says which of the two is missing.

AUDIT
fi

if [ "$1" -eq 1 ]; then
cat <<'NOTICE'

vigil is installed and is not running yet.

  1. vigild configure --dry-run  — ask THIS host what it can watch, and print the
     configuration for it without writing anything. `vigild configure --force` then
     writes /etc/vigil/vigil.yaml and a file in /etc/vigil/collectors for each
     collector that can run here, keeping every file it replaces as .previous;
  2. read /etc/vigil/vigil.yaml and /etc/vigil/collectors — every value in them is
     already the default, so the files this package installed work as they stand;
  3. systemctl enable --now vigild
  4. vigil ui                    — the console, once the daemon is up

Reading the nftables ruleset is done by vigil-firewall.timer, which vigild.service pulls in:
/usr/sbin/nft runs there and not inside the agent, so the agent keeps the two capabilities it
had. To stop that reading and keep everything else:  systemctl mask vigil-firewall.timer

What this host's container engines hold is read the same way, by vigil-containers.timer, which
runs /usr/sbin/vigil-container-dump. To stop that one:  systemctl mask vigil-containers.timer

NOTICE
for collector in firewall containers-engines; do
    if /usr/sbin/vigild collector "$collector" enable > /tmp/vigil-enable.$$ 2>&1; then
        sed 's/^/  /' /tmp/vigil-enable.$$
    else
        echo "vigil: the $collector collector was left switched off on this host:"
        sed 's/^/  /' /tmp/vigil-enable.$$
        echo "vigil: switch it on when that is fixed:  vigild collector $collector enable"
    fi
    rm -f /tmp/vigil-enable.$$
done
else
echo
echo "vigil: this is an upgrade, so /etc/vigil/vigil.yaml was not touched. To switch"
echo "vigil: the firewall collector on:  vigild collector firewall enable"
echo "vigil: the container engines on:   vigild collector containers-engines enable"
echo
if ! grep -q '^collectors_path:' /etc/vigil/vigil.yaml 2>/dev/null; then
    echo "vigil: /etc/vigil/vigil.yaml names its collectors itself and is read as it always was."
    echo "vigil: Each collector now has a file of its own in /etc/vigil/collectors. To move to"
    echo "vigil: them, take collectors, schedule, interval_seconds, killing, accounts, units and"
    echo "vigil: every collector's block out of vigil.yaml, carry what you changed into those"
    echo "vigil: files, and add to vigil.yaml:"
    echo "vigil:   collectors_path: /etc/vigil/collectors"
    for kept in suppressions reporters; do
        grep -q "^${kept}_path:" /etc/vigil/vigil.yaml 2>/dev/null \
            || echo "vigil:   ${kept}_path: /etc/vigil/$kept"
    done
    echo "vigil: \`vigild configure --dry-run\` prints the whole layout for this host."
    echo
fi
fi

%preun
if [ "$1" -eq 0 ] && [ -d /run/systemd/system ]; then
    systemctl --no-reload disable vigild.service vigil-firewall.timer vigil-containers.timer >/dev/null 2>&1 || true
    systemctl stop vigild.service vigil-firewall.timer vigil-containers.timer >/dev/null 2>&1 || true
fi

%postun
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
fi

if [ "$1" -eq 0 ]; then
    if command -v auditctl >/dev/null 2>&1; then
        echo "vigil: the vigil_exec audit rule stays loaded in the kernel until the rules are"
        echo "vigil: reloaded:  augenrules --load  (and restart auditd to drop the plugin)."
    fi

    kept=""
    [ -d /var/lib/vigil ] && kept="$kept /var/lib/vigil"
    [ -d /var/log/vigil ] && kept="$kept /var/log/vigil"
    if [ -n "$kept" ]; then
        echo "vigil: kept on purpose:$kept"
        echo "vigil: findings are the record of what happened on this host."
        echo "vigil: remove them deliberately if you want them gone:  rm -rf$kept"
    fi
fi

%changelog
