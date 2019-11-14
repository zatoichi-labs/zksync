package manager

import (
	"context"
	"errors"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"time"

	"github.com/digitalocean/godo"
	"golang.org/x/oauth2"
	v1 "k8s.io/api/apps/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/clientcmd"
)

const defaultTimeout = 15 * time.Second

// Manager manages number of provers and nodes in a provers nodepool.
type Manager struct {
	deploymentName string
	k8sClientset   *kubernetes.Clientset
}

// New returns new manager instance.
func New(token, clusterID, deploymentName string) (*Manager, error) {
	if token == "" || clusterID == "" || deploymentName == "" {
		return nil, errors.New("wrong arguments")
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTimeout)
	defer cancel()
	doClient := newDOClient(ctx, token)

	if v, res, err := doClient.Kubernetes.GetKubeConfig(ctx, clusterID); err != nil {
		log.Printf("[Manager.Add]: %v\n", err)
		return nil, fmt.Errorf("could not get kubeconfig: %v", err)
	} else if res.StatusCode != http.StatusOK {
		log.Printf("[Manager.Add]: %v", res.Status)
		return nil, fmt.Errorf("could not get kubeconfig, status: %d", res.StatusCode)
	} else if kubeconfigFile, err := createKubeconfigFile(v.KubeconfigYAML); err != nil {
		return nil, fmt.Errorf("could not create kubeconfig: %v", err)
	} else if k8sClientset, err := createK8sClientset(kubeconfigFile); err != nil {
		return nil, fmt.Errorf("could not create k8s client: %v", err)
	} else {
		return &Manager{
			deploymentName: deploymentName,
			k8sClientset:   k8sClientset,
		}, nil
	}
}

// Add increases number of replicas by one.
func (m *Manager) Add() error {
	deployment, err := m.getDeployment()
	if err != nil {
		return err
	}
	*deployment.Spec.Replicas++
	return m.updateDeployment(deployment)
}

// Cut decreases number of replicas by one.
func (m *Manager) Cut() error {
	deployment, err := m.getDeployment()
	if err != nil {
		return err
	}
	*deployment.Spec.Replicas--
	return m.updateDeployment(deployment)
}

func (m *Manager) getDeployment() (*v1.Deployment, error) {
	d, err := m.k8sClientset.AppsV1().Deployments("default").Get(m.deploymentName, metav1.GetOptions{})
	if err != nil {
		return nil, err
	}
	if d.Spec.Replicas == nil {
		v := int32(0)
		d.Spec.Replicas = &v
	}
	return d, nil
}

func (m *Manager) updateDeployment(deployment *v1.Deployment) error {
	_, err := m.k8sClientset.AppsV1().Deployments("default").Update(deployment)
	return err
}

type oathToken struct {
	v string
}

func (t *oathToken) Token() (*oauth2.Token, error) {
	token := &oauth2.Token{
		AccessToken: t.v,
	}
	return token, nil
}

func newDOClient(ctx context.Context, token string) *godo.Client {
	oauthClient := oauth2.NewClient(ctx, &oathToken{
		v: token,
	})
	return godo.NewClient(oauthClient)
}

func createKubeconfigFile(yaml []byte) (string, error) {
	if f, err := ioutil.TempFile("", ""); err != nil {
		return "", fmt.Errorf("could not create kubeconfig file: %v", err)
	} else if _, err := f.Write(yaml); err != nil {
		return "", fmt.Errorf("could not write kubeconfig file: %v", err)
	} else if err := f.Close(); err != nil {
		return "", fmt.Errorf("could not persist kubeconfig file changes: %v", err)
	} else {
		return f.Name(), nil
	}
}

func createK8sClientset(kubeconfig string) (*kubernetes.Clientset, error) {
	// use the current context in kubeconfig
	config, err := clientcmd.BuildConfigFromFlags("", kubeconfig)
	if err != nil {
		return nil, err
	}

	return kubernetes.NewForConfig(config)
}
