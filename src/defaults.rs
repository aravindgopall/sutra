use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultsCatalog {
    pub families: Vec<DefaultsFamily>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultsFamily {
    pub base: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub subcommands: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

pub fn defaults_path() -> Option<PathBuf> {
    let mut p = home_dir()?;
    p.push(".sutra");
    std::fs::create_dir_all(&p).ok();
    p.push("defaults.json");
    Some(p)
}

pub fn load_defaults_catalog() -> DefaultsCatalog {
    let Some(p) = defaults_path() else {
        return DefaultsCatalog { families: vec![] };
    };
    if !p.exists() {
        return DefaultsCatalog { families: vec![] };
    }
    let Ok(data) = fs::read_to_string(p) else {
        return DefaultsCatalog { families: vec![] };
    };
    serde_json::from_str(&data).unwrap_or(DefaultsCatalog { families: vec![] })
}

pub fn generate_defaults_catalog(output: Option<PathBuf>, overwrite: bool) -> anyhow::Result<()> {
    let catalog = create_exhaustive_defaults_catalog();
    
    let output_path = output.or_else(defaults_path).ok_or_else(|| anyhow::anyhow!("Could not determine output path"))?;
    
    if output_path.exists() && !overwrite {
        return Err(anyhow::anyhow!("Defaults file already exists at {:?}. Use --overwrite to replace it.", output_path));
    }
    
    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(&catalog)?;
    fs::write(&output_path, json)?;
    
    Ok(())
}

fn create_exhaustive_defaults_catalog() -> DefaultsCatalog {
    DefaultsCatalog {
        families: vec![
            // Version control
            DefaultsFamily {
                base: "git".to_string(),
                aliases: vec!["g".to_string()],
                subcommands: vec![
                    "add".to_string(), "am".to_string(), "annotate".to_string(), "apply".to_string(),
                    "archive".to_string(), "bisect".to_string(), "blame".to_string(), "branch".to_string(),
                    "bundle".to_string(), "checkout".to_string(), "cherry".to_string(), "citool".to_string(),
                    "clean".to_string(), "clone".to_string(), "commit".to_string(), "config".to_string(),
                    "count-objects".to_string(), "describe".to_string(), "diff".to_string(), "difftool".to_string(),
                    "fetch".to_string(), "format-patch".to_string(), "gc".to_string(), "grep".to_string(),
                    "gui".to_string(), "help".to_string(), "init".to_string(), "instaweb".to_string(),
                    "log".to_string(), "merge".to_string(), "mv".to_string(), "notes".to_string(),
                    "pull".to_string(), "push".to_string(), "range-diff".to_string(), "rebase".to_string(),
                    "repack".to_string(), "replace".to_string(), "request-pull".to_string(), "reset".to_string(),
                    "restore".to_string(), "revert".to_string(), "rm".to_string(), "shortlog".to_string(),
                    "show".to_string(), "stash".to_string(), "status".to_string(), "submodule".to_string(),
                    "switch".to_string(), "tag".to_string(), "worktree".to_string(),
                ],
                patterns: vec![
                    "git add --all".to_string(), "git add -A".to_string(), "git add .".to_string(),
                    "git commit -m".to_string(), "git commit --amend".to_string(), "git push origin".to_string(),
                    "git push -u origin".to_string(), "git pull origin".to_string(), "git fetch origin".to_string(),
                    "git checkout -b".to_string(), "git checkout main".to_string(), "git checkout master".to_string(),
                    "git merge".to_string(), "git rebase".to_string(), "git status".to_string(), "git log".to_string(),
                    "git diff".to_string(), "git stash".to_string(), "git stash pop".to_string(), "git stash apply".to_string(),
                    "git branch".to_string(), "git branch -d".to_string(), "git branch -D".to_string(),
                    "git remote add origin".to_string(), "git remote set-url origin".to_string(),
                ],
            },
            
            // Container management
            DefaultsFamily {
                base: "docker".to_string(),
                aliases: vec!["d".to_string()],
                subcommands: vec![
                    "build".to_string(), "commit".to_string(), "cp".to_string(), "create".to_string(),
                    "diff".to_string(), "events".to_string(), "exec".to_string(), "export".to_string(),
                    "history".to_string(), "images".to_string(), "import".to_string(), "info".to_string(),
                    "inspect".to_string(), "kill".to_string(), "load".to_string(), "login".to_string(),
                    "logout".to_string(), "logs".to_string(), "pause".to_string(), "port".to_string(),
                    "ps".to_string(), "pull".to_string(), "push".to_string(), "rename".to_string(),
                    "restart".to_string(), "rm".to_string(), "rmi".to_string(), "run".to_string(),
                    "save".to_string(), "search".to_string(), "start".to_string(), "stats".to_string(),
                    "stop".to_string(), "tag".to_string(), "top".to_string(), "unpause".to_string(),
                    "update".to_string(), "version".to_string(), "wait".to_string(),
                ],
                patterns: vec![
                    "docker run -it".to_string(), "docker run --rm".to_string(), "docker run -d".to_string(),
                    "docker build -t".to_string(), "docker pull".to_string(), "docker push".to_string(),
                    "docker ps".to_string(), "docker ps -a".to_string(), "docker images".to_string(),
                    "docker rmi".to_string(), "docker rm".to_string(), "docker exec -it".to_string(),
                    "docker logs".to_string(), "docker stop".to_string(), "docker start".to_string(),
                    "docker restart".to_string(), "docker kill".to_string(), "docker system prune".to_string(),
                    "docker volume ls".to_string(), "docker network ls".to_string(), "docker-compose up".to_string(),
                    "docker-compose down".to_string(), "docker-compose build".to_string(),
                ],
            },
            
            // Kubernetes
            DefaultsFamily {
                base: "kubectl".to_string(),
                aliases: vec!["k".to_string(), "kub".to_string()],
                subcommands: vec![
                    "annotate".to_string(), "api-resources".to_string(), "api-versions".to_string(),
                    "apply".to_string(), "attach".to_string(), "auth".to_string(), "autoscale".to_string(),
                    "certificate".to_string(), "cluster-info".to_string(), "completion".to_string(),
                    "config".to_string(), "convert".to_string(), "cordon".to_string(), "cp".to_string(),
                    "create".to_string(), "delete".to_string(), "describe".to_string(), "diff".to_string(),
                    "drain".to_string(), "edit".to_string(), "exec".to_string(), "explain".to_string(),
                    "expose".to_string(), "get".to_string(), "kustomize".to_string(), "label".to_string(),
                    "logs".to_string(), "patch".to_string(), "plugin".to_string(), "port-forward".to_string(),
                    "proxy".to_string(), "replace".to_string(), "scale".to_string(), "set".to_string(),
                    "taint".to_string(), "top".to_string(), "uncordon".to_string(), "version".to_string(),
                    "wait".to_string(),
                ],
                patterns: vec![
                    "kubectl get pods".to_string(), "kubectl get services".to_string(), "kubectl get deployments".to_string(),
                    "kubectl get nodes".to_string(), "kubectl get namespaces".to_string(), "kubectl describe pod".to_string(),
                    "kubectl logs".to_string(), "kubectl exec -it".to_string(), "kubectl apply -f".to_string(),
                    "kubectl delete pod".to_string(), "kubectl delete deployment".to_string(), "kubectl create deployment".to_string(),
                    "kubectl port-forward".to_string(), "kubectl scale deployment".to_string(), "kubectl rollout status".to_string(),
                    "kubectl rollout restart".to_string(), "kubectl label pod".to_string(), "kubectl annotate pod".to_string(),
                    "kubectl config use-context".to_string(), "kubectl config current-context".to_string(),
                ],
            },
            
            // Package managers
            DefaultsFamily {
                base: "npm".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "i".to_string(), "uninstall".to_string(), "remove".to_string(),
                    "update".to_string(), "upgrade".to_string(), "list".to_string(), "ls".to_string(),
                    "search".to_string(), "info".to_string(), "init".to_string(), "publish".to_string(),
                    "version".to_string(), "view".to_string(), "run".to_string(), "test".to_string(),
                    "start".to_string(), "stop".to_string(), "restart".to_string(), "cache".to_string(),
                    "config".to_string(), "set".to_string(), "get".to_string(), "login".to_string(),
                    "logout".to_string(), "whoami".to_string(), "audit".to_string(), "fund".to_string(),
                    "license".to_string(), "doctor".to_string(), "explain".to_string(),
                ],
                patterns: vec![
                    "npm install".to_string(), "npm i".to_string(), "npm install --save".to_string(),
                    "npm install --save-dev".to_string(), "npm install -g".to_string(), "npm uninstall".to_string(),
                    "npm update".to_string(), "npm run".to_string(), "npm start".to_string(), "npm test".to_string(),
                    "npm init".to_string(), "npm init -y".to_string(), "npm version".to_string(), "npm publish".to_string(),
                    "npm list".to_string(), "npm ls".to_string(), "npm cache clean".to_string(), "npm audit".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "yarn".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "add".to_string(), "remove".to_string(), "upgrade".to_string(), "install".to_string(),
                    "run".to_string(), "test".to_string(), "start".to_string(), "build".to_string(),
                    "init".to_string(), "config".to_string(), "cache".to_string(), "info".to_string(),
                    "list".to_string(), "why".to_string(), "licenses".to_string(), "version".to_string(),
                    "publish".to_string(), "unplug".to_string(),
                ],
                patterns: vec![
                    "yarn add".to_string(), "yarn remove".to_string(), "yarn upgrade".to_string(),
                    "yarn install".to_string(), "yarn run".to_string(), "yarn start".to_string(),
                    "yarn test".to_string(), "yarn build".to_string(), "yarn init".to_string(),
                    "yarn init -y".to_string(), "yarn version".to_string(), "yarn publish".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "pip".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "uninstall".to_string(), "list".to_string(), "show".to_string(),
                    "search".to_string(), "download".to_string(), "wheel".to_string(), "hash".to_string(),
                    "completion".to_string(), "debug".to_string(), "config".to_string(), "help".to_string(),
                ],
                patterns: vec![
                    "pip install".to_string(), "pip install --user".to_string(), "pip install -r".to_string(),
                    "pip uninstall".to_string(), "pip list".to_string(), "pip show".to_string(), "pip freeze".to_string(),
                    "pip search".to_string(), "pip download".to_string(), "pip install --upgrade".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "cargo".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "build".to_string(), "check".to_string(), "clean".to_string(), "doc".to_string(),
                    "new".to_string(), "init".to_string(), "run".to_string(), "test".to_string(),
                    "bench".to_string(), "install".to_string(), "uninstall".to_string(), "update".to_string(),
                    "search".to_string(), "publish".to_string(), "package".to_string(), "verify-project".to_string(),
                    "metadata".to_string(), "locate-project".to_string(), "vendor".to_string(), "yank".to_string(),
                ],
                patterns: vec![
                    "cargo build".to_string(), "cargo build --release".to_string(), "cargo check".to_string(),
                    "cargo test".to_string(), "cargo run".to_string(), "cargo new".to_string(), "cargo init".to_string(),
                    "cargo add".to_string(), "cargo remove".to_string(), "cargo update".to_string(), "cargo publish".to_string(),
                    "cargo install".to_string(), "cargo clean".to_string(), "cargo doc".to_string(),
                ],
            },
            
            // System commands
            DefaultsFamily {
                base: "ls".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "ls -la".to_string(), "ls -l".to_string(), "ls -a".to_string(), "ls -lh".to_string(),
                    "ls -ltr".to_string(), "ls -lt".to_string(), "ls -1".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "grep".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "grep -r".to_string(), "grep -i".to_string(), "grep -n".to_string(), "grep -v".to_string(),
                    "grep -E".to_string(), "grep -A".to_string(), "grep -B".to_string(), "grep -C".to_string(),
                    "grep --color=always".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "find".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "find . -name".to_string(), "find . -type".to_string(), "find . -mtime".to_string(),
                    "find . -size".to_string(), "find . -exec".to_string(), "find . -print".to_string(),
                    "find / -perm".to_string(), "find . -user".to_string(), "find . -group".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "awk".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "awk '{print".to_string(), "awk -F".to_string(), "awk '{print $1}'".to_string(),
                    "awk '/pattern/'".to_string(), "awk -v".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "sed".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "sed 's/".to_string(), "sed -i".to_string(), "sed -e".to_string(), "sed -n".to_string(),
                    "sed 'p'".to_string(), "sed '/pattern/d'".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "curl".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "curl -X".to_string(), "curl -H".to_string(), "curl -d".to_string(), "curl -o".to_string(),
                    "curl -O".to_string(), "curl -L".to_string(), "curl -I".to_string(), "curl -s".to_string(),
                    "curl -v".to_string(), "curl --user".to_string(), "curl --data".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "wget".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "wget -O".to_string(), "wget -c".to_string(), "wget -r".to_string(), "wget --user".to_string(),
                    "wget --password".to_string(), "wget -i".to_string(), "wget -q".to_string(),
                ],
            },
            
            // File operations
            DefaultsFamily {
                base: "cp".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "cp -r".to_string(), "cp -i".to_string(), "cp -v".to_string(), "cp -u".to_string(),
                    "cp -p".to_string(), "cp -a".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "mv".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "mv -i".to_string(), "mv -v".to_string(), "mv -u".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "rm".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "rm -r".to_string(), "rm -f".to_string(), "rm -rf".to_string(), "rm -i".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "mkdir".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "mkdir -p".to_string(), "mkdir -v".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "chmod".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "chmod 755".to_string(), "chmod 644".to_string(), "chmod +x".to_string(), "chmod -R".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "chown".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "chown user:group".to_string(), "chown -R".to_string(), "chown root".to_string(),
                ],
            },
            
            // Process management
            DefaultsFamily {
                base: "ps".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "ps aux".to_string(), "ps -ef".to_string(), "ps -f".to_string(), "ps -p".to_string(),
                    "ps -u".to_string(), "ps -o".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "kill".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "kill -9".to_string(), "kill -15".to_string(), "kill -TERM".to_string(), "kill -HUP".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "top".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "top -p".to_string(), "top -u".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "htop".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            // Network
            DefaultsFamily {
                base: "ssh".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "ssh user@host".to_string(), "ssh -p".to_string(), "ssh -i".to_string(), "ssh -L".to_string(),
                    "ssh -R".to_string(), "ssh -N".to_string(), "ssh -f".to_string(), "ssh -X".to_string(),
                    "ssh -Y".to_string(), "ssh -o".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "scp".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "scp file user@host:".to_string(), "scp -r".to_string(), "scp -P".to_string(),
                    "scp -i".to_string(), "scp user@host:file .".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "rsync".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "rsync -avz".to_string(), "rsync -r".to_string(), "rsync -u".to_string(), "rsync -delete".to_string(),
                    "rsync -e".to_string(), "rsync --exclude".to_string(), "rsync --include".to_string(),
                ],
            },
            
            // Compression
            DefaultsFamily {
                base: "tar".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "tar -czf".to_string(), "tar -xzf".to_string(), "tar -tf".to_string(), "tar -cvf".to_string(),
                    "tar -xvf".to_string(), "tar -tvf".to_string(), "tar -xzvf".to_string(), "tar -czvf".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gzip".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "gzip -d".to_string(), "gzip -c".to_string(), "gzip -9".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gunzip".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "bzip2".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "bzip2 -d".to_string(), "bzip2 -c".to_string(), "bzip2 -9".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "zip".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "zip -r".to_string(), "zip -e".to_string(), "zip -u".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "unzip".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "unzip -l".to_string(), "unzip -d".to_string(),
                ],
            },
            
            // Text processing
            DefaultsFamily {
                base: "cat".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "cat file".to_string(), "cat -n".to_string(), "cat -b".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "head".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "head -n".to_string(), "head -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "tail".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "tail -n".to_string(), "tail -f".to_string(), "tail -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "sort".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "sort -n".to_string(), "sort -r".to_string(), "sort -u".to_string(), "sort -k".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "uniq".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "uniq -c".to_string(), "uniq -d".to_string(), "uniq -u".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "wc".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "wc -l".to_string(), "wc -w".to_string(), "wc -c".to_string(), "wc -m".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "cut".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "cut -d".to_string(), "cut -f".to_string(), "cut -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "tr".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "tr 'a-z' 'A-Z'".to_string(), "tr -d".to_string(), "tr -s".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "paste".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "paste -d".to_string(), "paste -s".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "join".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "join -t".to_string(), "join -a".to_string(), "join -v".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "diff".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "diff -u".to_string(), "diff -r".to_string(), "diff -w".to_string(), "diff -i".to_string(),
                    "diff -B".to_string(), "diff -b".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "patch".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "patch -p1".to_string(), "patch -p0".to_string(), "patch -R".to_string(),
                ],
            },
            
            // Development tools
            DefaultsFamily {
                base: "make".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "make all".to_string(), "make clean".to_string(), "make install".to_string(),
                    "make test".to_string(), "make dist".to_string(), "make help".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gcc".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "gcc -o".to_string(), "gcc -Wall".to_string(), "gcc -g".to_string(), "gcc -O2".to_string(),
                    "gcc -I".to_string(), "gcc -L".to_string(), "gcc -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "g++".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "g++ -o".to_string(), "g++ -Wall".to_string(), "g++ -g".to_string(), "g++ -O2".to_string(),
                    "g++ -I".to_string(), "g++ -L".to_string(), "g++ -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "clang".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "clang -o".to_string(), "clang -Wall".to_string(), "clang -g".to_string(), "clang -O2".to_string(),
                    "clang -I".to_string(), "clang -L".to_string(), "clang -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "clang++".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "clang++ -o".to_string(), "clang++ -Wall".to_string(), "clang++ -g".to_string(), "clang++ -O2".to_string(),
                    "clang++ -I".to_string(), "clang++ -L".to_string(), "clang++ -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "python".to_string(),
                aliases: vec!["python3".to_string()],
                subcommands: vec![],
                patterns: vec![
                    "python script.py".to_string(), "python -m".to_string(), "python -c".to_string(),
                    "python -i".to_string(), "python -v".to_string(), "python -h".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "node".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "node script.js".to_string(), "node -e".to_string(), "node -p".to_string(),
                    "node -i".to_string(), "node -v".to_string(), "node -h".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "java".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "java -jar".to_string(), "java -cp".to_string(), "java -version".to_string(),
                    "java -Xmx".to_string(), "java -Xms".to_string(), "java -D".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "javac".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "javac -cp".to_string(), "javac -d".to_string(), "javac -g".to_string(),
                    "javac -sourcepath".to_string(), "javac -bootclasspath".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "mvn".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "clean".to_string(), "compile".to_string(), "test".to_string(), "package".to_string(),
                    "install".to_string(), "deploy".to_string(), "site".to_string(), "dependency".to_string(),
                    "surefire".to_string(), "failsafe".to_string(), "exec".to_string(), "versions".to_string(),
                ],
                patterns: vec![
                    "mvn clean".to_string(), "mvn compile".to_string(), "mvn test".to_string(), "mvn package".to_string(),
                    "mvn install".to_string(), "mvn deploy".to_string(), "mvn clean install".to_string(),
                    "mvn clean compile".to_string(), "mvn clean test".to_string(), "mvn clean package".to_string(),
                    "mvn dependency:tree".to_string(), "mvn dependency:resolve".to_string(), "mvn exec:java".to_string(),
                    "mvn versions:display-dependency-updates".to_string(), "mvn versions:display-plugin-updates".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gradle".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "build".to_string(), "clean".to_string(), "test".to_string(), "install".to_string(),
                    "assemble".to_string(), "check".to_string(), "dependencies".to_string(), "projects".to_string(),
                    "tasks".to_string(), "wrapper".to_string(), "help".to_string(),
                ],
                patterns: vec![
                    "gradle build".to_string(), "gradle clean".to_string(), "gradle test".to_string(),
                    "gradle install".to_string(), "gradle assemble".to_string(), "gradle check".to_string(),
                    "gradle dependencies".to_string(), "gradle projects".to_string(), "gradle tasks".to_string(),
                    "gradle wrapper".to_string(), "gradle help".to_string(),
                ],
            },
            
            // Package managers (Linux)
            DefaultsFamily {
                base: "apt".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "update".to_string(), "upgrade".to_string(), "install".to_string(), "remove".to_string(),
                    "autoremove".to_string(), "autoclean".to_string(), "search".to_string(), "show".to_string(),
                    "list".to_string(), "edit-sources".to_string(),
                ],
                patterns: vec![
                    "apt update".to_string(), "apt upgrade".to_string(), "apt install".to_string(), "apt remove".to_string(),
                    "apt autoremove".to_string(), "apt autoclean".to_string(), "apt search".to_string(), "apt show".to_string(),
                    "apt list".to_string(), "apt edit-sources".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "apt-get".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "update".to_string(), "upgrade".to_string(), "install".to_string(), "remove".to_string(),
                    "autoremove".to_string(), "autoclean".to_string(), "search".to_string(), "show".to_string(),
                    "list".to_string(), "clean".to_string(),
                ],
                patterns: vec![
                    "apt-get update".to_string(), "apt-get upgrade".to_string(), "apt-get install".to_string(), "apt-get remove".to_string(),
                    "apt-get autoremove".to_string(), "apt-get autoclean".to_string(), "apt-get search".to_string(), "apt-get show".to_string(),
                    "apt-get list".to_string(), "apt-get clean".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "yum".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "remove".to_string(), "update".to_string(), "search".to_string(),
                    "list".to_string(), "info".to_string(), "clean".to_string(), "provides".to_string(),
                    "groupinstall".to_string(), "groupremove".to_string(),
                ],
                patterns: vec![
                    "yum install".to_string(), "yum remove".to_string(), "yum update".to_string(), "yum search".to_string(),
                    "yum list".to_string(), "yum info".to_string(), "yum clean".to_string(), "yum provides".to_string(),
                    "yum groupinstall".to_string(), "yum groupremove".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "dnf".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "remove".to_string(), "update".to_string(), "search".to_string(),
                    "list".to_string(), "info".to_string(), "clean".to_string(), "provides".to_string(),
                    "groupinstall".to_string(), "groupremove".to_string(),
                ],
                patterns: vec![
                    "dnf install".to_string(), "dnf remove".to_string(), "dnf update".to_string(), "dnf search".to_string(),
                    "dnf list".to_string(), "dnf info".to_string(), "dnf clean".to_string(), "dnf provides".to_string(),
                    "dnf groupinstall".to_string(), "dnf groupremove".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "pacman".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "remove".to_string(), "update".to_string(), "sync".to_string(),
                    "search".to_string(), "list".to_string(), "info".to_string(), "clean".to_string(),
                    "query".to_string(), "upgrade".to_string(),
                ],
                patterns: vec![
                    "pacman -S".to_string(), "pacman -R".to_string(), "pacman -Syu".to_string(), "pacman -Ss".to_string(),
                    "pacman -Sl".to_string(), "pacman -Si".to_string(), "pacman -Sc".to_string(), "pacman -Qs".to_string(),
                    "pacman -U".to_string(),
                ],
            },
            
            // macOS specific
            DefaultsFamily {
                base: "brew".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "uninstall".to_string(), "update".to_string(), "upgrade".to_string(),
                    "search".to_string(), "list".to_string(), "info".to_string(), "cleanup".to_string(),
                    "services".to_string(), "doctor".to_string(), "config".to_string(), "tap".to_string(),
                ],
                patterns: vec![
                    "brew install".to_string(), "brew uninstall".to_string(), "brew update".to_string(), "brew upgrade".to_string(),
                    "brew search".to_string(), "brew list".to_string(), "brew info".to_string(), "brew cleanup".to_string(),
                    "brew services".to_string(), "brew doctor".to_string(), "brew config".to_string(), "brew tap".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "port".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "install".to_string(), "uninstall".to_string(), "update".to_string(), "upgrade".to_string(),
                    "search".to_string(), "list".to_string(), "info".to_string(), "clean".to_string(),
                    "variants".to_string(), "deps".to_string(),
                ],
                patterns: vec![
                    "port install".to_string(), "port uninstall".to_string(), "port update".to_string(), "port upgrade".to_string(),
                    "port search".to_string(), "port list".to_string(), "port info".to_string(), "port clean".to_string(),
                    "port variants".to_string(), "port deps".to_string(),
                ],
            },
            
            // Cloud/DevOps
            DefaultsFamily {
                base: "aws".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "s3".to_string(), "ec2".to_string(), "lambda".to_string(), "iam".to_string(),
                    "cloudformation".to_string(), "cloudwatch".to_string(), "dynamodb".to_string(),
                    "rds".to_string(), "sqs".to_string(), "sns".to_string(), "iam".to_string(),
                    "configure".to_string(), "sts".to_string(), "logs".to_string(), "elbv2".to_string(),
                ],
                patterns: vec![
                    "aws s3 cp".to_string(), "aws s3 ls".to_string(), "aws s3 sync".to_string(),
                    "aws ec2 describe-instances".to_string(), "aws ec2 run-instances".to_string(),
                    "aws lambda invoke".to_string(), "aws lambda list-functions".to_string(),
                    "aws iam create-user".to_string(), "aws iam list-users".to_string(),
                    "aws cloudformation create-stack".to_string(), "aws cloudformation describe-stacks".to_string(),
                    "aws configure".to_string(), "aws sts get-caller-identity".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gcloud".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "compute".to_string(), "storage".to_string(), "functions".to_string(), "iam".to_string(),
                    "projects".to_string(), "config".to_string(), "auth".to_string(), "services".to_string(),
                    "sql".to_string(), "container".to_string(), "pubsub".to_string(), "datastore".to_string(),
                ],
                patterns: vec![
                    "gcloud compute instances list".to_string(), "gcloud compute instances create".to_string(),
                    "gcloud storage cp".to_string(), "gcloud storage ls".to_string(),
                    "gcloud functions deploy".to_string(), "gcloud functions list".to_string(),
                    "gcloud iam service-accounts create".to_string(), "gcloud iam service-accounts list".to_string(),
                    "gcloud projects list".to_string(), "gcloud config set".to_string(),
                    "gcloud auth login".to_string(), "gcloud auth list".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "az".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "storage".to_string(), "vm".to_string(), "functionapp".to_string(), "ad".to_string(),
                    "group".to_string(), "configure".to_string(), "login".to_string(), "account".to_string(),
                    "webapp".to_string(), "container".to_string(), "network".to_string(), "monitor".to_string(),
                ],
                patterns: vec![
                    "az storage account create".to_string(), "az storage account list".to_string(),
                    "az vm create".to_string(), "az vm list".to_string(),
                    "az functionapp create".to_string(), "az functionapp list".to_string(),
                    "az ad sp create-for-rbac".to_string(), "az ad sp list".to_string(),
                    "az group create".to_string(), "az group list".to_string(),
                    "az configure".to_string(), "az login".to_string(), "az account list".to_string(),
                ],
            },
            
            // Monitoring/Debugging
            DefaultsFamily {
                base: "strace".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "strace -p".to_string(), "strace -e".to_string(), "strace -o".to_string(),
                    "strace -f".to_string(), "strace -tt".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "lsof".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "lsof -i".to_string(), "lsof -p".to_string(), "lsof -u".to_string(),
                    "lsof -n".to_string(), "lsof -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "netstat".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "netstat -tuln".to_string(), "netstat -an".to_string(), "netstat -i".to_string(),
                    "netstat -r".to_string(), "netstat -p".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "ss".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "ss -tuln".to_string(), "ss -an".to_string(), "ss -i".to_string(),
                    "ss -r".to_string(), "ss -p".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "iostat".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "iostat -x".to_string(), "iostat -d".to_string(), "iostat -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "vmstat".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "vmstat 1".to_string(), "vmstat -s".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "free".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "free -h".to_string(), "free -m".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "df".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "df -h".to_string(), "df -i".to_string(), "df -T".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "du".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "du -h".to_string(), "du -sh".to_string(), "du -h --max-depth=1".to_string(),
                ],
            },
            
            // System administration
            DefaultsFamily {
                base: "systemctl".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "start".to_string(), "stop".to_string(), "restart".to_string(), "reload".to_string(),
                    "enable".to_string(), "disable".to_string(), "status".to_string(), "list-units".to_string(),
                    "list-unit-files".to_string(), "is-active".to_string(), "is-enabled".to_string(),
                    "mask".to_string(), "unmask".to_string(), "daemon-reload".to_string(),
                ],
                patterns: vec![
                    "systemctl start".to_string(), "systemctl stop".to_string(), "systemctl restart".to_string(),
                    "systemctl enable".to_string(), "systemctl disable".to_string(), "systemctl status".to_string(),
                    "systemctl list-units".to_string(), "systemctl list-unit-files".to_string(),
                    "systemctl is-active".to_string(), "systemctl is-enabled".to_string(), "systemctl mask".to_string(),
                    "systemctl unmask".to_string(), "systemctl daemon-reload".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "journalctl".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "journalctl -f".to_string(), "journalctl -u".to_string(), "journalctl --since".to_string(),
                    "journalctl --until".to_string(), "journalctl -p".to_string(), "journalctl -n".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "crontab".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "crontab -e".to_string(), "crontab -l".to_string(), "crontab -r".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "useradd".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "useradd -m".to_string(), "useradd -s".to_string(), "useradd -G".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "usermod".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "usermod -aG".to_string(), "usermod -s".to_string(), "usermod -L".to_string(),
                    "usermod -U".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "passwd".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "passwd username".to_string(), "passwd -l".to_string(), "passwd -u".to_string(),
                ],
            },
            
            // Text editors
            DefaultsFamily {
                base: "vim".to_string(),
                aliases: vec!["vi".to_string()],
                subcommands: vec![],
                patterns: vec![
                    "vim file".to_string(), "vim +".to_string(), "vim -O".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "nano".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "nano file".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "emacs".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "emacs file".to_string(), "emacs -nw".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "tmux".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "new".to_string(), "attach".to_string(), "detach".to_string(), "list-sessions".to_string(),
                    "kill-session".to_string(), "kill-server".to_string(), "new-window".to_string(),
                    "split-window".to_string(), "select-window".to_string(), "send-keys".to_string(),
                ],
                patterns: vec![
                    "tmux new -s".to_string(), "tmux attach -t".to_string(), "tmux detach".to_string(),
                    "tmux list-sessions".to_string(), "tmux kill-session -t".to_string(), "tmux kill-server".to_string(),
                    "tmux new-window".to_string(), "tmux split-window".to_string(), "tmux select-window".to_string(),
                    "tmux send-keys".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "screen".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "screen -S".to_string(), "screen -r".to_string(), "screen -ls".to_string(),
                    "screen -d".to_string(), "screen -x".to_string(),
                ],
            },
            
            // Development servers
            DefaultsFamily {
                base: "python".to_string(),
                aliases: vec!["python3".to_string()],
                subcommands: vec![],
                patterns: vec![
                    "python -m http.server".to_string(), "python -m SimpleHTTPServer".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "http-server".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "http-server -p".to_string(), "http-server -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "live-server".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "live-server --port".to_string(), "live-server --open".to_string(),
                ],
            },
            
            // Database
            DefaultsFamily {
                base: "mysql".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "mysql -u".to_string(), "mysql -h".to_string(), "mysql -P".to_string(),
                    "mysql -p".to_string(), "mysql -e".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "psql".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "psql -U".to_string(), "psql -h".to_string(), "psql -p".to_string(),
                    "psql -d".to_string(), "psql -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "sqlite3".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "sqlite3 database.db".to_string(), "sqlite3 -line".to_string(), "sqlite3 -header".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "redis-cli".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "redis-cli -h".to_string(), "redis-cli -p".to_string(), "redis-cli -n".to_string(),
                ],
            },
            
            // Build tools
            DefaultsFamily {
                base: "cmake".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "cmake .".to_string(), "cmake -B".to_string(), "cmake --build".to_string(),
                    "cmake -DCMAKE_BUILD_TYPE=Release".to_string(), "cmake -DCMAKE_INSTALL_PREFIX=".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "bazel".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "build".to_string(), "test".to_string(), "run".to_string(), "clean".to_string(),
                    "query".to_string(), "info".to_string(), "version".to_string(),
                ],
                patterns: vec![
                    "bazel build".to_string(), "bazel test".to_string(), "bazel run".to_string(),
                    "bazel clean".to_string(), "bazel query".to_string(), "bazel info".to_string(),
                    "bazel version".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "gradle".to_string(),
                aliases: vec![],
                subcommands: vec![
                    "build".to_string(), "clean".to_string(), "test".to_string(), "install".to_string(),
                    "assemble".to_string(), "check".to_string(), "dependencies".to_string(), "projects".to_string(),
                    "tasks".to_string(), "wrapper".to_string(), "help".to_string(),
                ],
                patterns: vec![
                    "gradle build".to_string(), "gradle clean".to_string(), "gradle test".to_string(),
                    "gradle install".to_string(), "gradle assemble".to_string(), "gradle check".to_string(),
                    "gradle dependencies".to_string(), "gradle projects".to_string(), "gradle tasks".to_string(),
                    "gradle wrapper".to_string(), "gradle help".to_string(),
                ],
            },
            
            // Testing
            DefaultsFamily {
                base: "pytest".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "pytest".to_string(), "pytest -v".to_string(), "pytest -s".to_string(),
                    "pytest -k".to_string(), "pytest --cov".to_string(), "pytest --html".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "rspec".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "rspec".to_string(), "rspec spec/".to_string(), "rspec --format".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "jest".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "jest".to_string(), "jest --watch".to_string(), "jest --coverage".to_string(),
                    "jest --testNamePattern".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "mocha".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "mocha".to_string(), "mocha --watch".to_string(), "mocha --grep".to_string(),
                    "mocha --reporter".to_string(),
                ],
            },
            
            // Documentation
            DefaultsFamily {
                base: "man".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "man command".to_string(), "man -k".to_string(), "man -f".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "info".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "info command".to_string(),
                ],
            },
            
            // Miscellaneous
            DefaultsFamily {
                base: "watch".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "watch -n".to_string(), "watch -d".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "timeout".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "timeout 10s".to_string(), "timeout -k".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "xargs".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "xargs -I".to_string(), "xargs -n".to_string(), "xargs -P".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "printf".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "printf '%s\n'".to_string(), "printf '%d\n'".to_string(), "printf '%x\n'".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "echo".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "echo 'text'".to_string(), "echo -e".to_string(), "echo -n".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "date".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "date +%Y-%m-%d".to_string(), "date +%H:%M:%S".to_string(), "date -d".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "cal".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "cal 2023".to_string(), "cal 12 2023".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "bc".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "bc -l".to_string(), "bc <<<".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "dc".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "dc <<<".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "expr".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "expr 1 + 1".to_string(), "expr length".to_string(), "expr substr".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "test".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "test -f".to_string(), "test -d".to_string(), "test -e".to_string(),
                    "test -x".to_string(), "test -z".to_string(), "test -n".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "true".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "false".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "yes".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "yes".to_string(), "yes y".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "env".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "env | grep".to_string(), "env -i".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "export".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "export VAR=value".to_string(), "export -p".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "alias".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "alias name='command'".to_string(), "alias -p".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "unalias".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "unalias name".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "history".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "history".to_string(), "history | grep".to_string(), "history -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "fc".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "fc -l".to_string(), "fc -s".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "bind".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "bind -l".to_string(), "bind -p".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "complete".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "complete -W".to_string(), "complete -F".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "compgen".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "compgen -c".to_string(), "compgen -f".to_string(), "compgen -d".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "shopt".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "shopt -s".to_string(), "shopt -u".to_string(), "shopt -o".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "getopts".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "shift".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "eval".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "eval command".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "exec".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "exec command".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "source".to_string(),
                aliases: vec![".".to_string()],
                subcommands: vec![],
                patterns: vec![
                    "source file".to_string(), ". file".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "exit".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "exit 0".to_string(), "exit 1".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "logout".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![],
            },
            
            DefaultsFamily {
                base: "return".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "return 0".to_string(), "return 1".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "break".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "break".to_string(), "break 2".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "continue".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "continue".to_string(), "continue 2".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "trap".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "trap command".to_string(), "trap -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "ulimit".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "ulimit -n".to_string(), "ulimit -u".to_string(), "ulimit -c".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "umask".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "umask 022".to_string(), "umask -S".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "wait".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "wait".to_string(), "wait $!".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "jobs".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "jobs".to_string(), "jobs -l".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "fg".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "fg %1".to_string(), "fg %job".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "bg".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "bg %1".to_string(), "bg %job".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "kill".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "kill -9".to_string(), "kill -15".to_string(), "kill -TERM".to_string(), "kill -HUP".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "killall".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "killall process".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "pkill".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "pkill process".to_string(), "pkill -f".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "pgrep".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "pgrep process".to_string(), "pgrep -f".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "nohup".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "nohup command &".to_string(),
                ],
            },
            
            DefaultsFamily {
                base: "disown".to_string(),
                aliases: vec![],
                subcommands: vec![],
                patterns: vec![
                    "disown %1".to_string(),
                ],
            },
        ],
    }
}
