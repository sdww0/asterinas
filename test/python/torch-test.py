import torch
import numpy as np
from torch.utils.data import DataLoader
from torchvision import transforms
from torchvision import datasets
import torch.multiprocessing as mp
import torchvision.models as models
import time
import torch.nn.functional as F

# Super parameter ------------------------------------------------------------------------------------
batch_size = 64
learning_rate = 0.005
momentum = 0.9
EPOCH = 5
# Prepare dataset ------------------------------------------------------------------------------------
normalize = transforms.Normalize(
    mean=[0.4914, 0.4822, 0.4465], std=[0.2023, 0.1994, 0.2010]
)
transform = transforms.Compose(
    [transforms.Resize(227), transforms.ToTensor(), normalize]
)
train_dataset = datasets.CIFAR10(root="./data/cifar", train=True, transform=transform)
test_dataset = datasets.CIFAR10(root="./data/cifar", train=False, transform=transform)
train_loader = DataLoader(train_dataset, batch_size=batch_size, shuffle=True)
test_loader = DataLoader(test_dataset, batch_size=batch_size, shuffle=False)
# Construct loss and optimizer ------------------------------------------------------------------------------
model = models.alexnet(pretrained=False)

criterion = torch.nn.CrossEntropyLoss()
optimizer = torch.optim.SGD(model.parameters(), lr=learning_rate, momentum=momentum)


# Train and Test CLASS --------------------------------------------------------------------------------------
def train(epoch):
    running_loss = 0.0
    running_total = 0
    running_correct = 0
    for batch_idx, data in enumerate(train_loader, 0):
        inputs, target = data
        optimizer.zero_grad()

        outputs = model(inputs)
        loss = criterion(outputs, target)
        loss.backward()
        optimizer.step()

        running_loss += loss.item()
        _, predicted = torch.max(outputs.data, dim=1)
        running_total += inputs.shape[0]
        running_correct += (predicted == target).sum().item()

        print("pass one batch, id: %d" % (batch_idx))

        if batch_idx % 300 == 299:
            # print(
            #     "[epoch: %d, batch index: %5d]: loss: %.3f , acc: %.2f %%"
            #     % (
            #         epoch + 1,
            #         batch_idx + 1,
            #         running_loss / 300,
            #         100 * running_correct / running_total,
            #     )
            # )
            running_loss = 0.0
            running_total = 0
            running_correct = 0


def test(epoch):
    correct = 0
    total = 0
    with torch.no_grad():
        for data in test_loader:
            images, labels = data
            outputs = model(images)
            _, predicted = torch.max(outputs.data, dim=1)
            total += labels.size(0)
            correct += (predicted == labels).sum().item()
    acc = correct / total
    print("[%d / %d]: Accuracy on test set: %.1f %% " % (epoch + 1, EPOCH, 100 * acc))


def start_train_test():
    for epoch in range(EPOCH):
        train(epoch)
        test(epoch)
        print("Epoch: {} time: {}".format(epoch, time.asctime()))


# Start train and Test --------------------------------------------------------------------------------------
if __name__ == "__main__":
    num_processes = mp.cpu_count()
    processes = []
    # model.share_memory();

    print("num process: {}".format(num_processes))
    print("start time: {}".format(time.asctime()))

    for rank in range(num_processes):
        p = mp.Process(target=start_train_test, args=())
        p.start()
        processes.append(p)

    for p in processes:
        p.join()
    print("end time: {}".format(time.asctime()))
